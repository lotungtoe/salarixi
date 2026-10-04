use azalea::pathfinder::PathfinderClientExt;

use crate::bot::extensions::{BotDefaultExt, BotMovementExt};
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::BotStatus;
use crate::bot::systems::states::STATE_SYSTEM;
use crate::bot::systems::tasks::TASK_SYSTEM;
use crate::bot::PLUGINS;
use crate::server::transfer::*;
use crate::{sleep, take_bot, take_profile};

pub async fn reset_bot(index: &u8, silent: bool) {
  TASK_SYSTEM.kill_all_tasks_for(index).await;

  sleep!(600);

  take_bot!(index, async |bot| {
    if !bot.workable() {
      return;
    }

    bot.stop_move(index).await;
    bot.stop_crouching();
    bot.stop_jumping();
    bot.stop_pathfinding();
  });

  STATE_SYSTEM.reset(index).await;

  if !silent {
    let Some(username) = INDEX_SYSTEM.username_by_index(index).await else {
      return;
    };

    emit_log(
      format!("Все задачи и состояния бота {} сброшены", username),
      LogClass::Info,
    );

    emit_msg(
      "Управление",
      format!("Все задачи и состояния бота {} сброшены", username),
    );
  }
}

pub async fn disconnect_bot(index: &u8, silent: bool) {
  PLUGINS.kill_all_tasks_for(index).await;
  TASK_SYSTEM.kill_all_tasks_for(index).await;

  sleep!(1000);

  take_bot!(index, async |bot| {
    bot.disconnect();
  });

  STATE_SYSTEM.remove(index).await;
  TASK_SYSTEM.remove(index).await;

  take_profile!(index, async |profile| {
    profile.captcha_caught = false;
    profile.status = BotStatus::Disconnected;
  });

  if !silent {
    let Some(username) = INDEX_SYSTEM.username_by_index(index).await else {
      return;
    };

    emit_log(format!("Бот {} отключился", username), LogClass::Info);
    emit_msg("Управление", format!("Бот {} отключился", username));
  }
}
