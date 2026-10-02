use std::sync::Arc;

use azalea::registry::builtin::ItemKind;

use crate::bot::extensions::BotInventoryExt;
use crate::bot::plugin_manager::PluginName;
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::launch::options::AutoTotemOptions;
use crate::{sleep, take_bot};

pub struct AutoTotemPlugin;

impl AutoTotemPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8, options: &Arc<AutoTotemOptions>) {
    let mut inv_menu = None;

    take_bot!(index, async |bot| {
      inv_menu = bot.get_inventory_menu();
    });

    let Some(menu) = inv_menu else {
      return;
    };

    if !options.ignore_item_in_offhand {
      if let Some(offhand_item) = menu.slot(45) {
        if !offhand_item.is_empty() && offhand_item.kind() != ItemKind::Shield {
          sleep!(options.delay_after_taking as u64);
          return;
        }
      }
    }

    for (slot, item) in menu.slots().iter().enumerate() {
      if slot == 45 || item.kind() != ItemKind::TotemOfUndying {
        continue;
      }

      take_bot!(index, async |bot| {
        bot
          .inventory_move_item(index, ItemKind::TotemOfUndying, slot, 45, !options.ignore_states)
          .await;
      });

      sleep!(options.delay_after_taking as u64);
    }
  }
}

impl SalarixiPlugin<AutoTotemOptions> for AutoTotemPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8, options: Arc<AutoTotemOptions>) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index, &options).await;
        sleep!(options.tick_interval as u64);
      }
    });

    PLUGINS.push_task(&index, PluginName::AutoTotem, task);
  }
}
