use azalea::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::controllers::{get_state, State};
use crate::service::core::extensions::{BotDefaultExt, BotRotationExt, EntityType};
use crate::service::core::plugin_manager::PluginName;
use crate::service::core::registry::BOT_REGISTRY;
use crate::service::core::traits::SalarixiPlugin;
use crate::service::core::{process_is_active, PLUGIN_MANAGER};
use crate::service::generators::randuint;

pub struct AutoLookPlugin;

impl AutoLookPlugin {
  /// Метод поворота головы на ближайшую сущность
  async fn look(&self, bot: &Client) {
    let username = bot.name();

    if get_state(&username, State::CanLooking) && bot.is_goto_target_reached() {
      if let Some(entity) = bot.find_nearest_entity(EntityType::Any, 14.0) {
        bot.look_at_entity(entity, true);
        sleep(Duration::from_millis(randuint(50, 100))).await;
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
        if !process_is_active() {
          break;
        }

        let _ = BOT_REGISTRY
          .async_get_bot(&username_clone, async |bot| {
            if !bot.workable() {
              return;
            }

            self.look(bot).await;
          })
          .await;

        sleep(Duration::from_millis(50)).await;
      }
    });

    PLUGIN_MANAGER.push_root_task(&username, PluginName::AutoLookPlugin, task);
  }
}
