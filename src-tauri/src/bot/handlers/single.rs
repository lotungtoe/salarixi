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
use tokio::time::Duration;

use crate::api::emit::*;
use crate::bot::auth::{default_authorize, trigger_authorize};
use crate::bot::extensions::BotDefaultExt;
use crate::bot::registry::{RegistryEvent, BOT_REGISTRY};
use crate::bot::script::SCRIPT_EXECUTOR;
use crate::bot::systems::{BotStatus, PROFILE_SYSTEM, STATE_SYSTEM, TASK_SYSTEM};
use crate::bot::utils::acaptcha::{MAP_ACCUMULATOR, MAP_CAPTCHA_BYPASS, WEB_CAPTCHA_BYPASS};
use crate::bot::PLUGINS;
use crate::launch::process::current_options;
use crate::other::tools::*;
use crate::other::webhook::*;
use crate::take_bot;

/// Обработчик для одного бота
pub async fn single_handler(bot: Client, event: Event, _state: NoState) {
  match event {
    Event::Init => process_init(bot),
    Event::Spawn => process_spawn(bot).await,
    Event::Disconnect(text) => process_disconnect(bot, text).await,
    Event::Chat(packet) => process_chat_message(bot, packet).await,
    Event::Tick => process_tick(bot).await,
    Event::Packet(packet) => process_packet(bot, packet).await,
    _ => {}
  }
}

/// Функция обработки инициализации
fn process_init(bot: Client) {
  if let Some(opts) = current_options() {
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
}

/// Функция обработки спавна
async fn process_spawn(bot: Client) {
  let username = bot.username();

  BOT_REGISTRY.register_bot(&username, bot);
  TASK_SYSTEM.register(&username).await;
  STATE_SYSTEM.register(&username).await;

  BOT_REGISTRY.send_event(RegistryEvent::LoadPlugins {
    username: username.clone(),
  });

  PROFILE_SYSTEM.set_status(&username, BotStatus::Connected).await;

  let Some(opts) = current_options() else {
    return;
  };

  take_bot!(&username, async |bot| {
    let pos = bot.feet_pos();
    let health = bot.get_health();
    let str_pos = format!("{}, {}, {}", pos.x as i32, pos.y as i32, pos.z as i32);

    if opts.basic.use_webhook {
      send_webhook(
        opts.webhook.url,
        format!(
          "Бот {} заспавнился | Координаты (XYZ): {} | Здоровье: {} / 20",
          &username, str_pos, health
        ),
      );
    }

    emit_log(
      format!("Бот {} заспавнился, координаты (XYZ): {}", &username, str_pos),
      "info",
    );

    emit_msg(
      "Система",
      format!(
        "Бот {} заспавнился<br><br>Координаты (XYZ): {}<br>Здоровье: {} / 20",
        &username, str_pos, health
      ),
    );

    // Я без понятия что ставить в приоретет - сценарий пользователя или авторизацию.
    // Если я оберну авторизацию и сценарий в отдельные асинхронные задачи, то в таком
    // случае может возникнуть конфликт между ними. Наверное логичнее будет поставить
    // в приоретет авторизацию, ведь часто пользователь может использовать авто-скрипт
    // с включенной авторизацией, и если сценарий пользователя не будет выполнять процесс
    // авторизации, но при этом на сервере авторизация обязательна - бот просто будет ожидать
    // некоторое время в лобби, после его кикнет. Легче вперёд поставить авторизацию,
    // так как в большинстве случаев выполнения сценария предпологается после авторизации,
    // если что это всегда можно сделать иначе - просто отключить опции рода авто-регистр.
    default_authorize(&bot).await;

    if opts.basic.use_auto_script {
      if let Some(script) = opts.basic.script {
        // Здесь нужно указывать юзернейм бота, чтобы исполнитель работал только с ним
        SCRIPT_EXECUTOR.execute(bot.username(), script);
      }
    }
  });
}

/// Функция обработки отключения
async fn process_disconnect(bot: Client, text: Option<FormattedText>) {
  let username = bot.username();

  TASK_SYSTEM.kill_all_tasks_for(&username).await;
  PLUGINS.kill_all_tasks_for(&username).await;

  tokio::time::sleep(Duration::from_secs(2)).await;

  BOT_REGISTRY.remove_bot(&username).await;

  PROFILE_SYSTEM.set_bool(&username, "logined", false).await;
  PROFILE_SYSTEM.set_bool(&username, "captcha_caught", false).await;
  STATE_SYSTEM.reset(&username).await;
  TASK_SYSTEM.remove(&username).await;

  PROFILE_SYSTEM.set_status(&username, BotStatus::Disconnected).await;

  if let Some(t) = text {
    if let Some(options) = current_options() {
      if options.basic.use_webhook {
        send_webhook(
          options.webhook.url,
          format!("Бот {} отключился: {}", &username, t.to_html()),
        );
      }
    }

    emit_log(format!("Бот {} отключился: {}", &username, t.to_string()), "info");
    emit_msg("Система", format!("Бот {} отключился: {}", &username, t.to_string()));
  }
}

/// Функция обработки сообщения из чата
async fn process_chat_message(bot: Client, packet: ChatPacket) {
  let username = bot.username();

  let Some(options) = current_options() else {
    return;
  };

  send_optional_event(OptionalEmitEvent::Chat(ChatEventPayload {
    receiver: username.clone(),
    message: packet.message().to_html(),
  }));

  if options.basic.use_anti_captcha && options.captcha_bypass.captcha_type.as_str() == "web" {
    if let Some(url) = WEB_CAPTCHA_BYPASS.catch_url_from_message(
      packet.message().to_string(),
      options.captcha_bypass.regex.as_str(),
      options.captcha_bypass.required_url_part,
    ) {
      let Some(profile) = PROFILE_SYSTEM.get(&username).await else {
        return;
      };

      if !profile.captcha_caught {
        PROFILE_SYSTEM.set_bool(&username, "captcha_caught", true).await;

        if options.basic.use_webhook && options.webhook.send_information {
          send_webhook(
            options.webhook.url,
            format!("Бот {} получил ссылку на капчу: {}", username, url),
          );
        }

        emit_log(
          format!("[ Анти-Капча ]: Бот {} получил ссылку на капчу", username),
          "info",
        );

        send_optional_event(OptionalEmitEvent::AntiWebCaptcha(AntiWebCaptchaEventPayload {
          captcha_url: url,
          username: username,
        }));
      }
    }
  }

  trigger_authorize(&bot, packet.message().to_string()).await;
}

/// Функция обработки тика
async fn process_tick(bot: Client) {
  let username = bot.username();

  let Some(mut profile) = PROFILE_SYSTEM.get(&username).await else {
    return;
  };

  if profile.status == BotStatus::Connected {
    if !bot.workable() {
      return;
    }

    profile.ping = bot.ping();
    profile.health = bot.get_health() as u32;
  }
}

/// Функция обработки пакета
async fn process_packet(bot: Client, packet: Arc<ClientboundGamePacket>) {
  let username = bot.username();

  match &*packet {
    ClientboundGamePacket::AddEntity(entity_packet) => {
      if entity_packet.entity_type == EntityKind::ItemFrame || entity_packet.entity_type == EntityKind::GlowItemFrame {
        let pos = entity_packet.position;
        MAP_ACCUMULATOR.add_frame_position(&username, entity_packet.id.0 as i32, pos.x, pos.y, pos.z);
      }
    }
    ClientboundGamePacket::MapItemData(data) => {
      if let Some(options) = current_options() {
        if options.basic.use_anti_captcha && options.captcha_bypass.captcha_type.as_str() == "map" {
          let Some(map_patch) = &data.color_patch.0 else {
            return;
          };

          let Some(profile) = PROFILE_SYSTEM.get(&username).await else {
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

              MAP_ACCUMULATOR.add_map_data(
                &username,
                map_patch.width as u32,
                map_patch.height as u32,
                map_patch.map_colors.clone(),
                pos.x,
                pos.z,
                yaw,
              );

              if MAP_ACCUMULATOR.is_processing(&username) {
                return;
              }

              let maps = MAP_ACCUMULATOR.get_maps(&username).unwrap_or_default();

              if maps.len()
                < (options.captcha_bypass.number_of_columns * options.captcha_bypass.number_of_rows) as usize
              {
                return;
              }

              MAP_ACCUMULATOR.set_processing(&username, true);

              if let Some(combined_base64) = MAP_ACCUMULATOR.combine_all(&username) {
                PROFILE_SYSTEM.set_bool(&username, "captcha_caught", true).await;
                MAP_ACCUMULATOR.update_captcha_time(&username);

                if options.basic.use_webhook && options.webhook.send_information {
                  send_webhook(options.webhook.url, format!("Бот {} получил капчу с карты", username));
                }

                emit_log(
                  format!("[ Анти-Капча ]: Бот {} получил капчу с карты", username),
                  "info",
                );

                if options.captcha_bypass.solve_mode == "auto" {
                  MAP_CAPTCHA_BYPASS.solve_captcha(username.clone(), combined_base64, options.captcha_bypass);
                } else {
                  send_optional_event(OptionalEmitEvent::AntiMapCaptcha(AntiMapCaptchaEventPayload {
                    base64_code: combined_base64,
                    username: username.to_string(),
                  }));
                }

                MAP_ACCUMULATOR.clear_maps(&username);
              } else {
                MAP_ACCUMULATOR.clear_maps(&username);
              }
            } else {
              let base64_code = MAP_CAPTCHA_BYPASS.create_png_image(
                map_patch.width as u32,
                map_patch.height as u32,
                &map_patch.map_colors,
              );

              PROFILE_SYSTEM.set_bool(&username, "captcha_caught", true).await;
              MAP_ACCUMULATOR.update_captcha_time(&username);

              if options.basic.use_webhook && options.webhook.send_information {
                send_webhook(options.webhook.url, format!("Бот {} получил капчу с карты", username));
              }

              emit_log(
                format!("[ Анти-Капча ]: Бот {} получил капчу с карты", username),
                "info",
              );

              if options.captcha_bypass.solve_mode == "auto" {
                MAP_CAPTCHA_BYPASS.solve_captcha(username, base64_code, options.captcha_bypass);
              } else {
                send_optional_event(OptionalEmitEvent::AntiMapCaptcha(AntiMapCaptchaEventPayload {
                  base64_code,
                  username: username.to_string(),
                }));
              }
            }
          }
        }
      }
    }
    _ => {}
  }
}
