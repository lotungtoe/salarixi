use std::time::Duration;

use azalea::prelude::*;
use tokio::time::sleep;

use crate::bot::extensions::{BotDefaultExt, BotRotationExt, EntityType};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::{getst, State};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::other::tools::randnum;
use crate::take_bot;

pub struct AutoLookPlugin;

impl AutoLookPlugin {
  /// Метод обработки тика
  async fn tick(&self, username: &str) {
    if getst(&username, State::CanLooking).await {
      take_bot!(username, async |bot| {
        if !bot.is_goto_target_reached() {
          return;
        }

        if let Some(entity) = bot.find_nearest_entity(EntityType::Any, 14.0) {
          bot.look_at_entity(entity, true);
        }
      });

      sleep(Duration::from_millis(randnum(50, 100))).await;
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
        self.tick(&username_clone).await;
        sleep(Duration::from_millis(100)).await;
      }
    });

    PLUGINS.push_task(&username, PluginName::AutoLook, task);
  }
}
