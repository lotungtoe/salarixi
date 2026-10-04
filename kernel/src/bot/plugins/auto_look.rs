use std::sync::Arc;

use azalea::prelude::*;
use azalea::EntityRef;

use crate::bot::extensions::{BotDefaultExt, BotRotationExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::launch::options::AutoLookOptions;
use crate::tools::randnum;
use crate::{sleep, take_bot};

pub struct AutoLookPlugin;

impl AutoLookPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8, options: &AutoLookOptions) {
    if !getst(index, State::CanLooking).await {
      return;
    }

    setmst(index, StateName::Looking, true).await;

    take_bot!(index, async |bot| {
      if !bot.is_goto_target_reached() {
        return;
      }

      if let Some(entity) = self.find_nearest_entity(bot, options) {
        bot.look_at_entity(&entity, options.with_eye_height);
      }
    });

    sleep!(randnum(50, 100));

    setmst(index, StateName::Looking, false).await;
  }

  /// Метод поиска ближайшей сущности по указанному фильтру
  fn find_nearest_entity(&self, bot: &Client, options: &AutoLookOptions) -> Option<EntityRef> {
    for entity_filter in &options.include_entities {
      if let Some(entity) = bot.find_nearest_entity(entity_filter, options.distance) {
        return Some(entity);
      }
    }

    None
  }
}

impl SalarixiPlugin<AutoLookOptions> for AutoLookPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8, options: Arc<AutoLookOptions>) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index, &options).await;
        sleep!(options.tick_interval as u64);
      }
    });

    PLUGINS.push_task(&index, PluginName::AutoLook, task);
  }
}
