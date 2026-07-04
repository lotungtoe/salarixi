use azalea::inventory::components::{Damage, Enchantments, MaxDamage};
use azalea::inventory::ItemStack;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;

use crate::bot::extensions::{BotInteractExt, BotInventoryExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::{getst, setmst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::other::tools::*;
use crate::{sleep, take_bot};

/// Структура данных сломанного предмета
struct BrokenItem {
  slot: usize,
  kind: ItemKind,
}

pub struct AutoMendingPlugin;

impl AutoMendingPlugin {
  /// Метод обработки тика
  async fn tick(&self, username: &str) {
    let broken_items = self.find_broken_items(username).await;

    for broken_item in broken_items {
      if !getst(username, State::IsEating).await && !getst(username, State::IsDrinking).await {
        self.repair_item(username, broken_item).await;
      }
    }
  }

  /// Метод поиска и взятия в руку бутылок опыта
  async fn take_experience_bottles(&self, username: &str) -> Option<i32> {
    let mut count = None;

    take_bot!(username, async |bot| {
      if let Some(menu) = bot.get_inventory_menu() {
        for (slot, item) in menu.slots().iter().enumerate() {
          if item.kind() == ItemKind::ExperienceBottle {
            bot.take_item(slot, true).await;
            count = Some(item.count());
          }
        }
      }
    });

    count
  }

  /// Метод починки сломанного предмета
  async fn repair_item(&self, username: &str, broken_item: BrokenItem) {
    let Some(count) = self.take_experience_bottles(username).await else {
      return;
    };

    for _ in 0..=count {
      if !getst(username, State::CanInteracting).await || !getst(username, State::CanLooking).await {
        continue;
      }

      setmst(username, StateName::Interacting, true).await;
      setmst(username, StateName::Looking, true).await;

      if broken_item.slot != 45 && broken_item.slot > 8 {
        take_bot!(username, async |bot| {
          bot.take_item(broken_item.slot, false).await;
          sleep!(50);
          bot.move_item_to_offhand(broken_item.kind).await;
        });
      }

      let mut slot = 45;

      if broken_item.slot >= 5 && broken_item.slot <= 8 {
        slot = broken_item.slot;
      }

      let mut inv_menu = None;

      take_bot!(username, async |bot| {
        inv_menu = bot.get_inventory_menu();
      });

      let Some(menu) = inv_menu else {
        break;
      };

      if let Some(item) = menu.slot(slot) {
        let current_damage = self.get_current_item_damage(item);
        let max_durability = self.get_max_item_durability(item);

        if current_damage != 0 && max_durability - current_damage < max_durability / 2 {
          take_bot!(username, async |bot| {
            let direction = bot.direction();

            if direction.1 < 84.0 {
              bot.set_direction(direction.0, randnum(84.0, 90.0) as f32);
              sleep!(randnum(150, 200));
            }

            bot.start_use_held_item(InteractionHand::MainHand);
          });
        } else {
          setmst(username, StateName::Interacting, false).await;
          setmst(username, StateName::Looking, false).await;
          break;
        }
      }

      setmst(username, StateName::Interacting, false).await;
      setmst(username, StateName::Looking, false).await;

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
  async fn find_broken_items(&self, username: &str) -> Vec<BrokenItem> {
    let mut broken_items = vec![];

    take_bot!(username, async |bot| {
      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      for (slot, item) in menu.slots().iter().enumerate() {
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
      }
    });

    broken_items
  }
}

impl SalarixiPlugin for AutoMendingPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, username: String) {
    let username_clone = username.clone();
    let task = tokio::spawn(async move {
      loop {
        self.tick(&username_clone).await;
        sleep!(300);
      }
    });

    PLUGINS.push_task(&username, PluginName::AutoMending, task);
  }
}
