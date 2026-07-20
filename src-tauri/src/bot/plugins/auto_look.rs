use azalea::prelude::*;

use crate::bot::extensions::{BotDefaultExt, BotRotationExt, EntityType};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::states::{getst, State};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::other::tools::randnum;
use crate::{sleep, take_bot};

pub struct AutoLookPlugin;

impl AutoLookPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8) {
    if getst(index, State::CanLooking).await {
      take_bot!(index, async |bot| {
        if !bot.is_goto_target_reached() {
          return;
        }

        if let Some(entity) = bot.find_nearest_entity(EntityType::Any, 14.0) {
          bot.look_at_entity(index, entity, true, true).await;
        }
      });

      sleep!(randnum(50, 100));
    }
  }
}

impl SalarixiPlugin for AutoLookPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index).await;
        sleep!(100);
      }
    });

    PLUGINS.push_task(&index, PluginName::AutoLook, task);
  }
}
