use azalea::inventory::components::{Damage, Enchantments, MaxDamage};
use azalea::inventory::ItemStack;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::controllers::{get_state, set_mutual_states, State, StateName};
use crate::service::core::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt};
use crate::service::core::plugin_manager::PluginName;
use crate::service::core::traits::SalarixiPlugin;
use crate::service::core::{process_is_active, PLUGINS};
use crate::service::generators::*;
use crate::take_bot;

/// Структура данных сломанного предмета
struct BrokenItem {
  slot: usize,
  kind: ItemKind,
}

pub struct AutoRepairPlugin;

impl AutoRepairPlugin {
  /// Метод починки всех сломанных предметов
  async fn repair_items(&self, bot: &Client) {
    let broken_items = self.find_broken_items(bot);
    let nickname = bot.username();

    for broken_item in broken_items {
      if !get_state(&nickname, State::IsEating) && !get_state(&nickname, State::IsDrinking) {
        self.repair_item(bot, broken_item).await;
      }
    }
  }

  /// Метод поиска и взятия в руку бутылок опыта
  async fn take_experience_bottles(&self, bot: &Client) -> Option<i32> {
    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if item.kind() == ItemKind::ExperienceBottle {
          bot.take_item(slot, true).await;
          return Some(item.count());
        }
      }
    }

    None
  }

  /// Метод починки сломанного предмета
  async fn repair_item(&self, bot: &Client, broken_item: BrokenItem) {
    if let Some(count) = self.take_experience_bottles(bot).await {
      let nickname = bot.username();

      for _ in 0..=count {
        if !get_state(&nickname, State::CanInteracting) || !get_state(&nickname, State::CanLooking) {
          continue;
        }

        set_mutual_states(&nickname, StateName::Interacting, true);
        set_mutual_states(&nickname, StateName::Looking, true);

        if broken_item.slot != 45 && broken_item.slot > 8 {
          bot.take_item(broken_item.slot, false).await;
          sleep(Duration::from_millis(50)).await;
          bot.move_item_to_offhand(broken_item.kind);
          sleep(Duration::from_millis(randuint(50, 150))).await;
        }

        let mut slot = 45;

        if broken_item.slot >= 5 && broken_item.slot <= 8 {
          slot = broken_item.slot;
        }

        if let Some(menu) = bot.get_inventory_menu() {
          if let Some(item) = menu.slot(slot) {
            let current_damage = self.get_current_item_damage(item);
            let max_durability = self.get_max_item_durability(item);

            if current_damage != 0 && max_durability - current_damage < max_durability / 2 {
              let direction = bot.direction();

              if direction.1 < 84.0 {
                bot.set_direction(direction.0, randfloat(84.0, 90.0) as f32);
                sleep(Duration::from_millis(randuint(150, 200))).await;
              }

              bot.start_use_held_item(InteractionHand::MainHand);

              sleep(Duration::from_millis(randuint(200, 250))).await;
            } else {
              return;
            }
          }
        }

        set_mutual_states(&nickname, StateName::Interacting, false);
        set_mutual_states(&nickname, StateName::Looking, false);
      }

      sleep(Duration::from_millis(50)).await;
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
  fn find_broken_items(&self, bot: &Client) -> Vec<BrokenItem> {
    let mut broken_items = vec![];

    if let Some(menu) = bot.get_inventory_menu() {
      menu.slots().iter().enumerate().for_each(|(slot, item)| {
        if item.is_empty() {
          return;
        }

        let current_damage = self.get_current_item_damage(item);
        let max_durability = self.get_max_item_durability(item);

        if current_damage != 0 && max_durability - current_damage < max_durability / 2 {
          if self.item_has_mending(bot, item) {
            broken_items.push(BrokenItem {
              slot: slot,
              kind: item.kind(),
            });
          }
        }
      });
    }

    broken_items
  }
}

impl SalarixiPlugin for AutoRepairPlugin {
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

        take_bot!(&username_clone, async |bot| {
          if !bot.workable() {
            return;
          }

          self.repair_items(bot).await;
        });

        sleep(Duration::from_millis(50)).await;
      }
    });

    PLUGINS.push_root_task(&username, PluginName::AutoRepairPlugin, task);
  }
}
