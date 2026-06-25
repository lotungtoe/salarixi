use azalea::inventory::ItemStack;
use azalea::prelude::*;
use azalea::registry::builtin::ItemKind;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::extensions::{BotDefaultExt, BotInventoryExt, ClickMode};
use crate::service::core::plugin_manager::PluginName;
use crate::service::core::traits::SalarixiPlugin;
use crate::service::core::{process_is_active, PLUGINS};
use crate::take_bot;

/// Часть брони
#[derive(PartialEq)]
pub enum ArmorPart {
  Helmet,
  Chestplate,
  Leggings,
  Boots,
}

/// Структура информации об элементе брони
struct Armor {
  part: ArmorPart,
  slot: usize,
  priority: u8,
}

/// Структура полного набора брони
struct ArmorSet {
  helmet: Option<Armor>,
  chestplate: Option<Armor>,
  leggings: Option<Armor>,
  boots: Option<Armor>,
}

pub struct AutoArmorPlugin;

impl AutoArmorPlugin {
  /// Метод экипировки брони
  async fn equip_armor(&self, bot: &Client) {
    let mut armors = vec![];

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if slot > 8 {
          if let Some(armor) = self.is_armor(item, slot) {
            armors.push(armor);
          }
        }
      }
    }

    let armor_set = self.get_best_armor(bot, armors);

    if let Some(helmet) = armor_set.helmet {
      if self.is_armor_better(bot, &helmet) {
        self.equip(bot, helmet.slot, 5).await;
      }
    }

    if let Some(chestplate) = armor_set.chestplate {
      if self.is_armor_better(bot, &chestplate) {
        self.equip(bot, chestplate.slot, 6).await;
      }
    }

    if let Some(leggings) = armor_set.leggings {
      if self.is_armor_better(&bot, &leggings) {
        self.equip(bot, leggings.slot, 7).await;
      }
    }

    if let Some(boots) = armor_set.boots {
      if self.is_armor_better(&bot, &boots) {
        self.equip(bot, boots.slot, 8).await;
      }
    }
  }

  /// Метод экипировки определённого элемента брони
  async fn equip(&self, bot: &Client, armor_slot: usize, target_slot: usize) {
    if let Some(menu) = bot.get_inventory_menu() {
      if let Some(item) = menu.slot(target_slot) {
        if !item.is_empty() {
          if let Some(_) = bot.find_empty_slot_in_invenotry() {
            bot.inventory_click(target_slot, ClickMode::Shift, true).await;
            sleep(Duration::from_millis(50)).await;
          } else {
            return;
          }
        }
      }
    }

    bot.inventory_click(armor_slot, ClickMode::Shift, true).await;
  }

  /// Метод проверки предмета, является ли тот бронёй
  fn is_armor(&self, item: &ItemStack, slot: usize) -> Option<Armor> {
    return Some(match item.kind() {
      // Шлема
      ItemKind::TurtleHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: 0,
      },
      ItemKind::LeatherHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: 1,
      },
      ItemKind::GoldenHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: 2,
      },
      ItemKind::ChainmailHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: 3,
      },
      ItemKind::CopperHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: 4,
      },
      ItemKind::IronHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: 5,
      },
      ItemKind::DiamondHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: 6,
      },
      ItemKind::NetheriteHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: 7,
      },

      // Нагрудники
      ItemKind::LeatherChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: 0,
      },
      ItemKind::GoldenChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: 1,
      },
      ItemKind::ChainmailChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: 2,
      },
      ItemKind::CopperChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: 3,
      },
      ItemKind::IronChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: 4,
      },
      ItemKind::DiamondChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: 5,
      },
      ItemKind::NetheriteChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: 6,
      },

      // Поножи
      ItemKind::LeatherLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: 0,
      },
      ItemKind::GoldenLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: 1,
      },
      ItemKind::ChainmailLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: 2,
      },
      ItemKind::CopperLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: 3,
      },
      ItemKind::IronLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: 4,
      },
      ItemKind::DiamondLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: 5,
      },
      ItemKind::NetheriteLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: 6,
      },

      // Ботинки
      ItemKind::LeatherBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: 0,
      },
      ItemKind::GoldenBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: 1,
      },
      ItemKind::ChainmailBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: 2,
      },
      ItemKind::CopperBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: 3,
      },
      ItemKind::IronBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: 7,
      },
      ItemKind::DiamondBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: 5,
      },
      ItemKind::NetheriteBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: 6,
      },

      _ => return None,
    });
  }

  /// Метод получения лучшего элемента брони из списка
  fn get_best_armor(&self, bot: &Client, armors: Vec<Armor>) -> ArmorSet {
    let mut armor_set = ArmorSet {
      helmet: None,
      chestplate: None,
      leggings: None,
      boots: None,
    };

    for armor in armors {
      match armor.part {
        ArmorPart::Helmet => {
          if let Some(helmet) = &armor_set.helmet {
            if armor.priority <= helmet.priority {
              continue;
            }
          }

          if self.is_armor_better(bot, &armor) {
            armor_set.helmet = Some(armor);
          }
        }
        ArmorPart::Chestplate => {
          if let Some(chestplate) = &armor_set.chestplate {
            if armor.priority <= chestplate.priority {
              continue;
            }
          }

          if self.is_armor_better(bot, &armor) {
            armor_set.chestplate = Some(armor);
          }
        }
        ArmorPart::Leggings => {
          if let Some(leggings) = &armor_set.leggings {
            if armor.priority <= leggings.priority {
              continue;
            }
          }

          if self.is_armor_better(bot, &armor) {
            armor_set.leggings = Some(armor);
          }
        }
        ArmorPart::Boots => {
          if let Some(boots) = &armor_set.boots {
            if armor.priority <= boots.priority {
              continue;
            }
          }

          if self.is_armor_better(bot, &armor) {
            armor_set.boots = Some(armor);
          }
        }
      }
    }

    armor_set
  }

  /// Метод сравнения текущей брони с другой
  fn is_armor_better(&self, bot: &Client, armor: &Armor) -> bool {
    let current_armor_slot = match armor.part {
      ArmorPart::Helmet => 5,
      ArmorPart::Chestplate => 6,
      ArmorPart::Leggings => 7,
      ArmorPart::Boots => 8,
    };

    if let Some(menu) = bot.get_inventory_menu() {
      if let Some(item) = menu.slot(current_armor_slot) {
        if let Some(current_armor) = self.is_armor(item, current_armor_slot) {
          if armor.part == current_armor.part {
            return armor.priority > current_armor.priority;
          }
        } else {
          return true;
        }
      }
    }

    false
  }
}

impl SalarixiPlugin for AutoArmorPlugin {
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

          self.equip_armor(bot).await;
        });

        sleep(Duration::from_millis(200)).await;
      }
    });

    PLUGINS.pushrtsk(&username, PluginName::AutoArmor, task);
  }
}
