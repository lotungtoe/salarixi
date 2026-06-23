use std::time::Duration;

use azalea::pathfinder::PathfinderClientExt;

use crate::api::emit::*;
use crate::service::core::controllers::*;
use crate::service::core::extensions::{BotDefaultExt, BotMovementExt};
use crate::service::core::registry::BOT_REGISTRY;
use crate::service::core::PLUGINS;
use crate::take_bot;

/// Вспомогательная функция отправки сообщения в чат от бота
pub async fn send_message_from_bot(username: String, message: String) {
  take_bot!(&username, async |bot| bot.chat(message));
}

/// Вспомогательная функция, позволяющая сбросить все задачи и состояния бота
pub async fn reset_bot(username: String) {
  TASKS.kill_all_tasks_for(&username).await;

  tokio::time::sleep(Duration::from_secs(2)).await;

  take_bot!(&username, async |bot| {
    bot.stop_move();
    bot.stop_crouching();
    bot.stop_jumping();
    bot.stop_pathfinding();
  });

  STATES.reset(&username);

  emit_log(format!("Все задачи и состояния бота {} сброшены", username), "info");

  emit_msg(
    "Управление",
    format!("Все задачи и состояния бота {} сброшены", username),
  );
}

/// Вспомогательная функция отключения бота
pub async fn disconnect_bot(username: String) {
  PLUGINS.kill_all_tasks_for(&username).await;
  TASKS.kill_all_tasks_for(&username).await;

  tokio::time::sleep(Duration::from_secs(2)).await;

  if let Some(bot) = BOT_REGISTRY.remove_bot(&username).await {
    bot.disconnect();
  }

  STATES.remove(&username);
  TASKS.remove(&username);

  PROFILES.set_bool(&username, "captcha_caught", false);
  PROFILES.set_str(&username, "status", "Оффлайн");

  emit_log(format!("Бот {} отключился", username), "info");
  emit_msg("Управление", format!("Бот {} отключился", username));
}
