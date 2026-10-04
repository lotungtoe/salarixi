use std::sync::Arc;

use azalea::chat::ChatPacket;
use azalea::entity::HumanoidArm;
use azalea::prelude::*;
use azalea::protocol::common::client_information::ParticleStatus;
use azalea::protocol::connect::ConnectionError;
use azalea::protocol::packets::game::ClientboundGamePacket;
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
use crate::bot::utils::acaptcha::{MAP_CAPTCHA_BYPASS, WEB_CAPTCHA_BYPASS};
use crate::bot::PLUGINS;
use crate::launch::options::CaptchaType;
use crate::launch::process::current_options;
use crate::launch::process::update_process_status;
use crate::server::transfer::*;
use crate::tools::*;
use crate::webhook::*;
use crate::{sleep, take_profile};

pub async fn single_handler(bot: Client, event: Event, _state: NoState) {
  match event {
    Event::Init => process_init(bot).await,
    Event::Spawn => process_spawn(bot).await,
    Event::Disconnect(text) => process_disconnect(bot, text).await,
    Event::ConnectionFailed(error) => process_connection_failed(bot, error).await,
    Event::Chat(packet) => process_chat_message(bot, packet).await,
    Event::Tick => process_tick(bot).await,
    Event::Packet(packet) => process_packet(bot, packet).await,
    _ => {}
  }
}

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

  if opts.basic.use_webhook {
    send_webhook(opts.webhook.url, format!("Бот {} заспавнился", username));
  }

  emit_log(format!("Бот {} заспавнился", username), LogClass::Info);
  emit_msg("Система", format!("Бот {} заспавнился", username));

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

  REGISTRY_SYSTEM.remove_bot(&index);

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

    emit_log(
      format!("Бот {} отключился: {}", username, t.to_string()),
      LogClass::Info,
    );
    emit_msg("Система", format!("Бот {} отключился: {}", username, t.to_string()));
  }
}

async fn process_connection_failed(bot: Client, error: Arc<ConnectionError>) {
  let account = bot.account();
  let username = account.username();
  let str_err = error.to_string();

  if let Some(options) = current_options().await {
    if options.basic.use_webhook {
      send_webhook(
        options.webhook.url,
        format!("Бот {} не смог подключиться: {}", username, str_err),
      );
    }
  }

  let msg = format!("Ошибка подключения бота {}: {}", username, str_err);
  emit_log(&msg, LogClass::Error);
  emit_msg("Система", msg);
}

async fn process_chat_message(bot: Client, packet: ChatPacket) {
  let Some(opts) = current_options().await else {
    return;
  };

  if !opts.basic.chat_monitoring {
    return;
  }

  let account = bot.account();
  let username = account.username();
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  TRANSFER.emit(TransferEvent::BotChat(BotChatPayload {
    index,
    message: packet.message().to_html(),
  }));

  if opts.basic.use_anti_captcha && opts.captcha_bypass.captcha_type == CaptchaType::Web {
    if let Some(url) = WEB_CAPTCHA_BYPASS.catch_url_from_message(
      packet.message().to_string(),
      opts.captcha_bypass.regex.as_str(),
      opts.captcha_bypass.required_url_part,
    ) {
      let Some(profile) = PROFILE_SYSTEM.get(&index).await else {
        return;
      };

      if !profile.captcha_caught {
        take_profile!(&index, async |profile| {
          profile.captcha_caught = true;
        });

        if opts.basic.use_webhook && opts.webhook.send_information {
          send_webhook(
            opts.webhook.url,
            format!("Бот {} получил ссылку на капчу: {}", username, url),
          );
        }

        emit_log(format!("Бот {} получил ссылку на капчу", username), LogClass::Info);

        TRANSFER.emit(TransferEvent::AntiWebCaptcha(AntiWebCaptchaPayload {
          index,
          captcha_url: url,
        }));
      }
    }
  }

  trigger_authorize(&index, packet.message().to_string()).await;
}

async fn process_tick(bot: Client) {
  let account = bot.account();
  let username = account.username();
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  take_profile!(&index, async |profile| {
    if !bot.workable() {
      profile.ping = 0;
      profile.health = 0;
      return;
    }

    if profile.status == BotStatus::Connected {
      profile.ping = bot.ping();
      profile.health = bot.get_health() as u32;
    }
  });
}

async fn process_packet(bot: Client, packet: Arc<ClientboundGamePacket>) {
  let account = bot.account();
  let username = account.username();
  let Some(index) = INDEX_SYSTEM.index_by_username(&username).await else {
    return;
  };

  match &*packet {
    ClientboundGamePacket::AddEntity(p) => {
      MAP_CAPTCHA_BYPASS.process_frame(index, p).await;
    }
    ClientboundGamePacket::MapItemData(p) => {
      MAP_CAPTCHA_BYPASS
        .process_map_data(&bot, username.to_string(), index, p)
        .await;
    }
    _ => {}
  }
}
