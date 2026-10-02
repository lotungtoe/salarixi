use std::sync::Arc;

use azalea::inventory::components::{Damage, Enchantments, MaxDamage};
use azalea::inventory::ItemStack;
use azalea::prelude::*;
use azalea::registry::builtin::ItemKind;

use crate::bot::extensions::BotInventoryExt;
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::launch::options::AutoMendingOptions;
use crate::tools::*;
use crate::{sleep, take_bot};

struct BrokenItem {
  slot: usize,
  kind: ItemKind,
}

pub struct AutoMendingPlugin;

impl AutoMendingPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8, options: &AutoMendingOptions) {
    let broken_items = self.find_broken_items(index, options).await;

    for broken_item in broken_items {
      if !getst(index, State::IsEating).await && !getst(index, State::IsDrinking).await {
        self.repair_item(index, options, broken_item).await;
      }
    }
  }

  /// Метод поиска и взятия в руку бутылок опыта
  async fn take_experience_bottles(&self, index: &u8) -> Option<i32> {
    let mut count = None;

    take_bot!(index, async |bot| {
      if let Some(menu) = bot.get_inventory_menu() {
        for (slot, item) in menu.slots().iter().enumerate() {
          if item.kind() == ItemKind::ExperienceBottle {
            bot.take_item(index, slot, true).await;
            count = Some(item.count());
            break;
          }
        }
      }
    });

    count
  }

  /// Метод починки сломанного предмета
  async fn repair_item(&self, index: &u8, options: &AutoMendingOptions, broken_item: BrokenItem) {
    let Some(count) = self.take_experience_bottles(index).await else {
      return;
    };

    for _ in 0..count {
      if !getst(index, State::CanInteracting).await || !getst(index, State::CanLooking).await {
        continue;
      }

      setmst(index, StateName::Interacting, true).await;
      setmst(index, StateName::Looking, true).await;

      if broken_item.slot != 45 && broken_item.slot > 8 {
        take_bot!(index, async |bot| {
          bot
            .inventory_move_item(index, broken_item.kind, broken_item.slot, 45, false)
            .await;
        });
      }

      let mut slot = 45;

      if broken_item.slot >= 5 && broken_item.slot <= 8 {
        slot = broken_item.slot;
      }

      let mut inv_menu = None;

      take_bot!(index, async |bot| {
        inv_menu = bot.get_inventory_menu();
      });

      let Some(menu) = inv_menu else {
        break;
      };

      if let Some(item) = menu.slot(slot) {
        let current_damage = self.get_current_item_damage(item);
        let max_durability = self.get_max_item_durability(item);
        let current_durability = (max_durability - current_damage) as f32;
        let unbroken_item_durability = (max_durability as f32 / 100.0) * options.percent_of_unbroken_item_durability;

        if current_damage != 0 && current_durability < unbroken_item_durability {
          take_bot!(index, async |bot| {
            let direction = bot.direction();

            if direction.1 < 84.0 {
              bot.set_direction(direction.0, randnum(84.0, 90.0) as f32);
              sleep!(randnum(150, 200));
            }

            bot.start_use_item();
          });
        } else {
          setmst(index, StateName::Interacting, false).await;
          setmst(index, StateName::Looking, false).await;
          break;
        }
      }

      setmst(index, StateName::Interacting, false).await;
      setmst(index, StateName::Looking, false).await;

      sleep!(randnum(200, 250));
    }
  }

  /// Метод получения урона предмета
  fn get_current_item_damage(&self, item: &ItemStack) -> i32 {
    if let Some(damage) = item.get_component::<Damage>() {
      return damage.amount;
    }

    0
  }

  /// Метод получения максимальной прочности предмета
  fn get_max_item_durability(&self, item: &ItemStack) -> i32 {
    if let Some(damage) = item.get_component::<MaxDamage>() {
      return damage.amount;
    }

    0
  }

  /// Метод проверки, зачарован ли предмет на починку
  fn item_has_mending(&self, bot: &Client, item: &ItemStack) -> bool {
    if let Some(enchantments) = item.get_component::<Enchantments>() {
      for (enchantment, _) in &enchantments.levels {
        if let Some(id) = bot.resolve_registry_name(enchantment) {
          if id.to_string().contains("minecraft:mending") {
            return true;
          }
        }
      }
    }

    false
  }

  /// Метод поиска сломанных предметов в инвентаре
  async fn find_broken_items(&self, index: &u8, options: &AutoMendingOptions) -> Vec<BrokenItem> {
    let mut broken_items = vec![];

    take_bot!(index, async |bot| {
      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      for (slot, item) in menu.slots().iter().enumerate() {
        if item.is_empty() {
          return;
        }

        let current_damage = self.get_current_item_damage(item);
        let max_durability = self.get_max_item_durability(item);
        let current_durability = (max_durability - current_damage) as f32;
        let unbroken_item_durability = (max_durability as f32 / 100.0) * options.percent_of_unbroken_item_durability;

        if current_damage != 0 && current_durability < unbroken_item_durability {
          if self.item_has_mending(bot, item) {
            broken_items.push(BrokenItem {
              slot: slot,
              kind: item.kind(),
            });
          }
        }
      }
    });

    broken_items
  }
}

impl SalarixiPlugin<AutoMendingOptions> for AutoMendingPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8, options: Arc<AutoMendingOptions>) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index, &options).await;
        sleep!(options.tick_interval as u64);
      }
    });

    PLUGINS.push_task(&index, PluginName::AutoMending, task);
  }
}
