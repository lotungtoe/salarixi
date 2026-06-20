use crate::api::emit::*;
use crate::service::core::bot::*;
use crate::service::core::registry::BOT_REGISTRY;
use crate::service::core::PLUGIN_MANAGER;

/// Вспомогательная функция отправки сообщения в чат от бота
pub fn send_message_from_bot(username: String, message: String) {
  let Some(bot) = BOT_REGISTRY.get_bot(&username) else {
    return;
  };

  bot.chat(message);
}

/// Вспомогательная функция, позволяющая сбросить все задачи и состояния бота
pub fn reset_bot(username: String) {
  let Some(bot) = BOT_REGISTRY.get_bot(&username) else {
    return;
  };

  TASKS.reset(&username);
  STATES.reset(&username);

  bot.set_crouching(false);
  bot.set_jumping(false);

  emit_log(format!("Все задачи и состояния бота {} сброшены", username), "info");

  emit_msg(
    "Управление",
    format!("Все задачи и состояния бота {} сброшены", username),
  );
}

/// Вспомогательная функция отключения бота
pub fn disconnect_bot(username: String) {
  tokio::spawn(async move {
    TASKS.reset(&username);
    PLUGIN_MANAGER.destroy_all_tasks(&username);

    if let Some(bot) = BOT_REGISTRY.take_bot(&username).await {
      bot.disconnect();
    }

    PROFILES.set_bool(&username, "captcha_caught", false);
    STATES.reset(&username);
    TASKS.remove(&username);
    PROFILES.set_str(&username, "status", "Оффлайн");

    emit_log(format!("Бот {} отключился", username), "info");
    emit_msg("Управление", format!("Бот {} отключился", username));
  });
}
