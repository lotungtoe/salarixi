use std::time::Duration;

use azalea::registry::builtin::ItemKind;
use tokio::time::sleep;

use crate::bot::extensions::BotInventoryExt;
use crate::bot::plugin_manager::PluginName;
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::take_bot;

pub struct AutoTotemPlugin;

impl AutoTotemPlugin {
  /// Метод обработки тика
  async fn tick(&self, username: &str) {
    take_bot!(username, async |bot| {
      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      if let Some(offhand_item) = menu.slot(45) {
        if !offhand_item.is_empty() && offhand_item.kind() != ItemKind::Shield {
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
    });
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
        self.tick(&username_clone).await;
        sleep(Duration::from_millis(150)).await;
      }
    });

    PLUGINS.push_task(&username, PluginName::AutoTotem, task);
  }
}
