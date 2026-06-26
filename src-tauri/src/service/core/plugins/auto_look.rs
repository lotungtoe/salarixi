use azalea::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::extensions::{BotDefaultExt, BotRotationExt, EntityType};
use crate::service::core::plugin_manager::PluginName;
use crate::service::core::systems::{getst, State};
use crate::service::core::traits::SalarixiPlugin;
use crate::service::core::PLUGINS;
use crate::service::tools::randnum;
use crate::take_bot;

pub struct AutoLookPlugin;

impl AutoLookPlugin {
  /// Метод поворота головы на ближайшую сущность
  async fn look(&self, bot: &Client) {
    let username = bot.username();

    if getst(&username, State::CanLooking).await && bot.is_goto_target_reached() {
      if let Some(entity) = bot.find_nearest_entity(EntityType::Any, 14.0) {
        bot.look_at_entity(entity, true);
        sleep(Duration::from_millis(randnum(50, 100))).await;
      }
    }
  }
}

impl SalarixiPlugin for AutoLookPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, username: String) {
    let username_clone = username.clone();
    let task = tokio::spawn(async move {
      loop {
        take_bot!(&username_clone, async |bot| {
          if !bot.workable() {
            return;
          }

          self.look(bot).await;
        });

        sleep(Duration::from_millis(100)).await;
      }
    });

    PLUGINS.push_task(&username, PluginName::AutoLook, task);
  }
}
