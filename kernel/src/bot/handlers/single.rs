use std::sync::Arc;

use azalea::chat::ChatPacket;
use azalea::entity::HumanoidArm;
use azalea::entity::LookDirection;
use azalea::prelude::*;
use azalea::protocol::common::client_information::ParticleStatus;
use azalea::protocol::packets::game::ClientboundGamePacket;
use azalea::registry::builtin::EntityKind;
use azalea::FormattedText;
use azalea::{ClientInformation, NoState};

use crate::bot::auth::{default_authorize, trigger_authorize};
use crate::bot::extensions::BotDefaultExt;
use crate::bot::script::SCRIPT_EXECUTOR;
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::BotStatus;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::bot::systems::registry::REGISTRY_SYSTEM;
use crate::bot::systems::states::STATE_SYSTEM;
use crate::bot::systems::tasks::TASK_SYSTEM;
use crate::bot::utils::acaptcha::{MAP_CAPTCHA_BYPASS, MAP_COLLECTOR, WEB_CAPTCHA_BYPASS};
use crate::bot::PLUGINS;
use crate::launch::process::current_options;
use crate::launch::process::update_process_status;
use crate::server::transfer::*;
use crate::sleep;
use crate::tools::*;
use crate::webhook::*;
use crate::{take_bot, take_profile};

/// Обработчик для одного бота
pub async fn single_handler(bot: Client, event: Event, _state: NoState) {
  match event {
    Event::Init => process_init(bot).await,
    Event::Spawn => process_spawn(bot).await,
    Event::Disconnect(text) => process_disconnect(bot, text).await,
    Event::Chat(packet) => process_chat_message(bot, packet).await,
    Event::Tick => process_tick(bot).await,
    Event::Packet(packet) => process_packet(bot, packet).await,
    _ => {}
  }
}

/// Функция обработки инициализации
async fn process_init(bot: Client) {
  let Some(opts) = current_options().await else {
    return;
  };

  bot.set_client_information(ClientInformation {
    view_distance: opts.basic.view_distance,
    chat_colors: true,
    main_hand: if let Some(arm) = opts.basic.humanoid_arm {
      if arm.as_str() == "left" {
        HumanoidArm::Left
      } else {
        HumanoidArm::Right
      }
    } else {
      if randchance(0.5) {
        HumanoidArm::Left
      } else {
        HumanoidArm::Right
      }
    },
    particle_status: ParticleStatus::Minimal,
    ..Default::default()
  });
}

/// Функция обработки спавна
async fn process_spawn(bot: Client) {
  let username = bot.username();
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  if !REGISTRY_SYSTEM.bots.contains_key(&index) {
    REGISTRY_SYSTEM.register_bot(index, bot);
  }

  TASK_SYSTEM.register(index).await;
  STATE_SYSTEM.register(index).await;

  let Some(opts) = current_options().await else {
    return;
  };

  PLUGINS.activate_for(index, opts.plugins).await;

  take_profile!(&index, async |profile| {
    profile.logined = false;
    profile.captcha_caught = false;
    profile.status = BotStatus::Connected;
  });

  update_process_status().await;

  take_bot!(&index, async |bot| {
    let pos = bot.feet_pos();
    let health = bot.get_health();
    let str_pos = format!("{}, {}, {}", pos.x as i32, pos.y as i32, pos.z as i32);

    if opts.basic.use_webhook {
      send_webhook(
        opts.webhook.url,
        format!(
          "Бот {} заспавнился | Координаты (XYZ): {} | Здоровье: {} / 20",
          username, str_pos, health
        ),
      );
    }

    emit_log(
      format!("Бот {} заспавнился, координаты (XYZ): {}", username, str_pos),
      "info",
    );

    emit_msg(
      "Система",
      format!(
        "Бот {} заспавнился<br><br>Координаты (XYZ): {}<br>Здоровье: {} / 20",
        username, str_pos, health
      ),
    );
  });

  // Я без понятия что ставить в приоретет - сценарий пользователя или авторизацию.
  // Если я оберну авторизацию и сценарий в отдельные асинхронные задачи, то в таком
  // случае может возникнуть конфликт между ними. Наверное логичнее будет поставить
  // в приоретет авторизацию, ведь часто пользователь может использовать авто-скрипт
  // с включенной авторизацией, и если сценарий пользователя не будет выполнять процесс
  // авторизации, но при этом на сервере авторизация обязательна - бот просто будет ожидать
  // некоторое время в лобби, после его кикнет. Легче вперёд поставить авторизацию,
  // так как в большинстве случаев выполнения сценария предпологается после авторизации,
  // если что это всегда можно сделать иначе - просто отключить опции рода авто-регистр.
  default_authorize(&index).await;

  if opts.basic.use_auto_script {
    if let Some(script) = opts.basic.script {
      tokio::spawn(async move {
        // Здесь нужно указывать юзернейм бота, чтобы исполнитель работал только с ним
        SCRIPT_EXECUTOR.execute(username, index, script);
      });
    }
  }
}

/// Функция обработки отключения
async fn process_disconnect(bot: Client, text: Option<FormattedText>) {
  let username = bot.username();
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  TASK_SYSTEM.kill_all_tasks_for(&index).await;
  PLUGINS.kill_all_tasks_for(&index).await;

  sleep!(2000);

  STATE_SYSTEM.reset(&index).await;
  TASK_SYSTEM.remove(&index).await;

  take_profile!(&index, async |profile| {
    profile.logined = false;
    profile.captcha_caught = false;
    profile.status = BotStatus::Disconnected;
  });

  update_process_status().await;

  if let Some(t) = text {
    if let Some(options) = current_options().await {
      if options.basic.use_webhook {
        send_webhook(
          options.webhook.url,
          format!("Бот {} отключился: {}", username, t.to_html()),
        );
      }
    }

    emit_log(format!("Бот {} отключился: {}", username, t.to_string()), "info");
    emit_msg("Система", format!("Бот {} отключился: {}", username, t.to_string()));
  }
}

/// Функция обработки сообщения из чата
async fn process_chat_message(bot: Client, packet: ChatPacket) {
  let username = bot.username();
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  let Some(options) = current_options().await else {
    return;
  };

  TRANSFER.emit(TransferEvent::BotChat(BotChatPayload {
    receiver: username.clone(),
    message: packet.message().to_html(),
  }));

  if options.basic.use_anti_captcha && options.captcha_bypass.captcha_type.as_str() == "web" {
    if let Some(url) = WEB_CAPTCHA_BYPASS.catch_url_from_message(
      packet.message().to_string(),
      options.captcha_bypass.regex.as_str(),
      options.captcha_bypass.required_url_part,
    ) {
      let Some(profile) = PROFILE_SYSTEM.get(&index).await else {
        return;
      };

      if !profile.captcha_caught {
        take_profile!(&index, async |profile| {
          profile.captcha_caught = true;
        });

        if options.basic.use_webhook && options.webhook.send_information {
          send_webhook(
            options.webhook.url,
            format!("Бот {} получил ссылку на капчу: {}", username, url),
          );
        }

        emit_log(format!("Бот {} получил ссылку на капчу", username), "info");

        TRANSFER.emit(TransferEvent::AntiWebCaptcha(AntiWebCaptchaPayload {
          captcha_url: url,
          username: username.clone(),
        }));
      }
    }
  }

  trigger_authorize(&index, packet.message().to_string()).await;
}

/// Функция обработки тика
async fn process_tick(bot: Client) {
  let username = bot.username();
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  drop(username);

  take_profile!(&index, async |profile| {
    if profile.status == BotStatus::Connected {
      if !bot.workable() {
        return;
      }

      profile.ping = bot.ping();
      profile.health = bot.get_health() as u32;
    }
  });
}

/// Функция обработки пакета
async fn process_packet(bot: Client, packet: Arc<ClientboundGamePacket>) {
  let username = bot.username();
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  match &*packet {
    ClientboundGamePacket::AddEntity(entity_packet) => {
      if entity_packet.entity_type == EntityKind::ItemFrame || entity_packet.entity_type == EntityKind::GlowItemFrame {
        let pos = entity_packet.position;
        MAP_COLLECTOR
          .add_frame_position(index, entity_packet.id.0 as i32, pos.x, pos.y, pos.z)
          .await;
      }
    }
    ClientboundGamePacket::MapItemData(data) => {
      let Some(options) = current_options().await else {
        return;
      };

      if options.basic.use_anti_captcha && options.captcha_bypass.captcha_type.as_str() == "map" {
        let Some(map_patch) = &data.color_patch.0 else {
          return;
        };

        let Some(profile) = PROFILE_SYSTEM.get(&index).await else {
          return;
        };

        if !profile.captcha_caught {
          let is_frame = options.captcha_bypass.captcha_subtype.as_str() == "frame";

          if is_frame {
            let pos = bot.feet_pos();
            let yaw = if let Some(look_dir) = bot.get_component::<LookDirection>() {
              look_dir.y_rot()
            } else {
              0.0
            };

            MAP_COLLECTOR
              .add_map_data(
                index,
                map_patch.width as u32,
                map_patch.height as u32,
                map_patch.map_colors.clone(),
                pos.x,
                pos.z,
                yaw,
              )
              .await;

            if MAP_COLLECTOR.is_processing(&index).await {
              return;
            }

            let maps = MAP_COLLECTOR.get_maps(&index).await.unwrap_or_default();

            if maps.len() < (options.captcha_bypass.number_of_columns * options.captcha_bypass.number_of_rows) as usize
            {
              return;
            }

            MAP_COLLECTOR.set_processing(index, true).await;

            if let Some(combined_base64) = MAP_COLLECTOR.combine_all(&index).await {
              take_profile!(&index, async |profile| {
                profile.captcha_caught = true;
              });

              MAP_COLLECTOR.update_captcha_time(index).await;

              if options.basic.use_webhook && options.webhook.send_information {
                send_webhook(options.webhook.url, format!("Бот {} получил капчу с карты", username));
              }

              emit_log(format!("Бот {} получил капчу с карты", username), "info");

              if options.captcha_bypass.solve_mode == "auto" {
                MAP_CAPTCHA_BYPASS.solve_captcha(username.clone(), combined_base64, options.captcha_bypass);
              } else {
                TRANSFER.emit(TransferEvent::AntiMapCaptcha(AntiMapCaptchaPayload {
                  base64_code: combined_base64,
                  username: username.to_string(),
                }));
              }
            }

            MAP_COLLECTOR.clear_maps(&index).await;
          } else {
            let base64_code = MAP_CAPTCHA_BYPASS.create_png_image(
              map_patch.width as u32,
              map_patch.height as u32,
              &map_patch.map_colors,
            );

            take_profile!(&index, async |profile| {
              profile.captcha_caught = true;
            });

            MAP_COLLECTOR.update_captcha_time(index).await;

            if options.basic.use_webhook && options.webhook.send_information {
              send_webhook(options.webhook.url, format!("Бот {} получил капчу с карты", username));
            }

            emit_log(format!("Бот {} получил капчу с карты", username), "info");

            if options.captcha_bypass.solve_mode == "auto" {
              MAP_CAPTCHA_BYPASS.solve_captcha(username, base64_code, options.captcha_bypass);
            } else {
              TRANSFER.emit(TransferEvent::AntiMapCaptcha(AntiMapCaptchaPayload {
                base64_code,
                username: username.to_string(),
              }));
            }
          }
        }
      }
    }
    _ => {}
  }
}
