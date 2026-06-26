use azalea::prelude::*;
use azalea::registry::builtin::ItemKind;
use std::time::Duration;
use tokio::time::sleep;

use crate::bot::extensions::{BotDefaultExt, BotInventoryExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::take_bot;

pub struct AutoTotemPlugin;

impl AutoTotemPlugin {
  /// Метод обработки тика
  async fn tick(&self, bot: &Client) {
    if let Some(menu) = bot.get_inventory_menu() {
      if let Some(item) = menu.slot(45) {
        if !item.is_empty() && item.kind() != ItemKind::Shield {
          return;
        }
      }

      for (slot, item) in menu.slots().iter().enumerate() {
        if slot == 45 {
          continue;
        }

        if item.kind() == ItemKind::TotemOfUndying {
          bot.inventory_move_item(ItemKind::TotemOfUndying, slot, 45, true).await;
        }
      }
    }
  }
}

impl SalarixiPlugin for AutoTotemPlugin {
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

          self.tick(bot).await;
        });

        sleep(Duration::from_millis(50)).await;
      }
    });

    PLUGINS.push_task(&username, PluginName::AutoTotem, task);
  }
}
