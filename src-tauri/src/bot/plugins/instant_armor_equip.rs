use azalea::inventory::ItemStack;
use azalea::prelude::*;
use azalea::registry::builtin::ItemKind;

use crate::bot::extensions::{BotInventoryExt, BotPhysicsExt, ClickMode};
use crate::bot::plugin_manager::PluginName;
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::{sleep, take_bot};

/// Часть брони
#[derive(Default, PartialEq)]
pub enum ArmorPart {
  Helmet,
  #[default]
  Chestplate,
  Leggings,
  Boots,
}

/// Структура информации об элементе брони
#[derive(Default)]
struct Armor {
  part: ArmorPart,
  slot: usize,
  priority: ArmorPriority,
}

/// Информация о приоретете брони
#[derive(Default)]
struct ArmorPriority {
  material_rank: u8,
  max_durability: i32,
  current_durability: i32,
  has_mending: bool,
  has_binding_curse: bool,
  unbreaking: i32,
  protection: i32,
  aqua_affinity: i32,
  feather_falling: i32,
  blast_protection: i32,
  projectile_protection: i32,
  fire_protection: i32,
}

/// Структура полного набора брони
struct ArmorSet {
  helmet: Option<Armor>,
  chestplate: Option<Armor>,
  leggings: Option<Armor>,
  boots: Option<Armor>,
}

pub struct InstantArmorEquipPlugin;

impl InstantArmorEquipPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8) {
    take_bot!(index, async |bot| {
      let mut armors = vec![];

      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      for (slot, item) in menu.slots().iter().enumerate() {
        if slot > 8 {
          if let Some(armor) = self.extract_armor_info(bot, item, slot) {
            armors.push(armor);
          }
        }
      }

      if armors.is_empty() {
        return;
      }

      let armor_set = self.get_best_armor_set(bot, armors);

      if let Some(helmet) = armor_set.helmet {
        if self.is_armor_better_than_current(bot, &helmet) {
          self.equip(bot, index, helmet.slot, 5).await;
        }
      }

      if let Some(chestplate) = armor_set.chestplate {
        if self.is_armor_better_than_current(bot, &chestplate) {
          self.equip(bot, index, chestplate.slot, 6).await;
        }
      }

      if let Some(leggings) = armor_set.leggings {
        if self.is_armor_better_than_current(&bot, &leggings) {
          self.equip(bot, index, leggings.slot, 7).await;
        }
      }

      if let Some(boots) = armor_set.boots {
        if self.is_armor_better_than_current(&bot, &boots) {
          self.equip(bot, index, boots.slot, 8).await;
        }
      }
    });
  }

  /// Метод экипировки определённого элемента брони
  async fn equip(&self, bot: &Client, index: &u8, armor_slot: usize, target_slot: usize) {
    if let Some(menu) = bot.get_inventory_menu() {
      if let Some(item) = menu.slot(target_slot) {
        if !item.is_empty() {
          if let Some(_) = bot.find_empty_slot_in_invenotry() {
            bot.inventory_click(index, target_slot, ClickMode::Shift, true).await;
            sleep!(50);
          } else {
            return;
          }
        }
      }
    }

    bot.inventory_click(index, armor_slot, ClickMode::Shift, true).await;
  }

  /// Метод проверки предмета, является ли тот бронёй
  fn is_armor(&self, item: &ItemStack, slot: usize) -> Option<Armor> {
    Some(match item.kind() {
      // Шлема
      ItemKind::TurtleHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 0,
          ..Default::default()
        },
      },
      ItemKind::LeatherHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 1,
          ..Default::default()
        },
      },
      ItemKind::GoldenHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 2,
          ..Default::default()
        },
      },
      ItemKind::ChainmailHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 3,
          ..Default::default()
        },
      },
      ItemKind::CopperHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 4,
          ..Default::default()
        },
      },
      ItemKind::IronHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 5,
          ..Default::default()
        },
      },
      ItemKind::DiamondHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 6,
          ..Default::default()
        },
      },
      ItemKind::NetheriteHelmet => Armor {
        part: ArmorPart::Helmet,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 7,
          ..Default::default()
        },
      },

      // Нагрудники
      ItemKind::LeatherChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 0,
          ..Default::default()
        },
      },
      ItemKind::GoldenChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 1,
          ..Default::default()
        },
      },
      ItemKind::ChainmailChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 2,
          ..Default::default()
        },
      },
      ItemKind::CopperChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 3,
          ..Default::default()
        },
      },
      ItemKind::IronChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 4,
          ..Default::default()
        },
      },
      ItemKind::DiamondChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 5,
          ..Default::default()
        },
      },
      ItemKind::NetheriteChestplate => Armor {
        part: ArmorPart::Chestplate,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 6,
          ..Default::default()
        },
      },

      // Поножи
      ItemKind::LeatherLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 0,
          ..Default::default()
        },
      },
      ItemKind::GoldenLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 1,
          ..Default::default()
        },
      },
      ItemKind::ChainmailLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 2,
          ..Default::default()
        },
      },
      ItemKind::CopperLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 3,
          ..Default::default()
        },
      },
      ItemKind::IronLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 4,
          ..Default::default()
        },
      },
      ItemKind::DiamondLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 5,
          ..Default::default()
        },
      },
      ItemKind::NetheriteLeggings => Armor {
        part: ArmorPart::Leggings,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 6,
          ..Default::default()
        },
      },

      // Ботинки
      ItemKind::LeatherBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 0,
          ..Default::default()
        },
      },
      ItemKind::GoldenBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 1,
          ..Default::default()
        },
      },
      ItemKind::ChainmailBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 2,
          ..Default::default()
        },
      },
      ItemKind::CopperBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 3,
          ..Default::default()
        },
      },
      ItemKind::IronBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 4,
          ..Default::default()
        },
      },
      ItemKind::DiamondBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 5,
          ..Default::default()
        },
      },
      ItemKind::NetheriteBoots => Armor {
        part: ArmorPart::Boots,
        slot: slot,
        priority: ArmorPriority {
          material_rank: 6,
          ..Default::default()
        },
      },

      _ => return None,
    })
  }

  /// Метод извлечения информации о элементе брони
  fn extract_armor_info(&self, bot: &Client, item: &ItemStack, slot: usize) -> Option<Armor> {
    let mut armor = self.is_armor(item, slot)?;

    let Some(meta) = bot.extract_item_meta(item) else {
      return None;
    };

    armor.priority.max_durability = meta.max_durability;
    armor.priority.current_durability = meta.current_durability;

    for (id, level) in &meta.enchantments {
      let id_str = id.as_str();

      let priority = match id_str {
        "minecraft:protection" => Some(&mut armor.priority.protection),
        "minecraft:blast_protection" => Some(&mut armor.priority.blast_protection),
        "minecraft:fire_protection" => Some(&mut armor.priority.fire_protection),
        "minecraft:projectile_protection" => Some(&mut armor.priority.projectile_protection),
        "minecraft:feather_falling" => Some(&mut armor.priority.feather_falling),
        "minecraft:aqua_affinity" => Some(&mut armor.priority.aqua_affinity),
        "minecraft:unbreaking" => Some(&mut armor.priority.unbreaking),
        _ => None,
      };

      if let Some(p) = priority {
        *p = *level;
      }

      if id_str == "minecraft:mending" {
        armor.priority.has_mending = true;
      }

      if id_str == "minecraft:binding_curse" {
        armor.priority.has_binding_curse = true;
      }
    }

    Some(armor)
  }

  /// Метод получения лучшего набора брони из списка
  fn get_best_armor_set(&self, bot: &Client, armors: Vec<Armor>) -> ArmorSet {
    let mut armor_set = ArmorSet {
      helmet: None,
      chestplate: None,
      leggings: None,
      boots: None,
    };

    for armor in armors {
      let armor_part = match armor.part {
        ArmorPart::Helmet => &mut armor_set.helmet,
        ArmorPart::Chestplate => &mut armor_set.chestplate,
        ArmorPart::Leggings => &mut armor_set.leggings,
        ArmorPart::Boots => &mut armor_set.boots,
      };

      if let Some(current_best_armor) = armor_part {
        if self.compare_armors(bot, &armor, current_best_armor) != 0 {
          continue;
        }
      }

      if self.is_armor_better_than_current(bot, &armor) {
        *armor_part = Some(armor);
      }
    }

    armor_set
  }

  /// Метод сравнения текущей брони с другой
  fn is_armor_better_than_current(&self, bot: &Client, armor: &Armor) -> bool {
    let current_armor_slot = match armor.part {
      ArmorPart::Helmet => 5,
      ArmorPart::Chestplate => 6,
      ArmorPart::Leggings => 7,
      ArmorPart::Boots => 8,
    };

    let Some(menu) = bot.get_inventory_menu() else {
      return false;
    };

    let Some(item) = menu.slot(current_armor_slot) else {
      return false;
    };

    if let Some(current_armor) = self.extract_armor_info(bot, item, current_armor_slot) {
      if current_armor.priority.has_binding_curse {
        return false;
      }

      if self.compare_armors(bot, armor, &current_armor) != 0 {
        return false;
      }
    } else {
      if armor.part == ArmorPart::Helmet && armor.priority.material_rank < 6 && armor.priority.has_binding_curse {
        return false;
      } else if armor.part != ArmorPart::Helmet && armor.priority.material_rank < 5 && armor.priority.has_binding_curse
      {
        return false;
      }
    }

    true
  }

  /// Метод сравнения элементов брони. Возвращает 0, 1 или 2
  /// (если 0 - лучше `armor_one`, если 1 - лучше `armor_two`,
  /// если 2 - элементы брони одинаковые по характеристикам)
  fn compare_armors(&self, bot: &Client, armor_one: &Armor, armor_two: &Armor) -> u8 {
    // Если броня имеет чару "проклятье несъёмности" и при этом она ниже алмазной, то её сразу нужно отбрасывать
    if armor_two.part == ArmorPart::Helmet
      && armor_two.priority.material_rank < 6
      && armor_two.priority.has_binding_curse
      && !armor_one.priority.has_binding_curse
    {
      return 0;
    } else if armor_two.part != ArmorPart::Helmet
      && armor_two.priority.material_rank < 5
      && armor_two.priority.has_binding_curse
      && !armor_one.priority.has_binding_curse
    {
      return 0;
    }

    if armor_one.part == ArmorPart::Helmet
      && armor_one.priority.material_rank < 6
      && armor_one.priority.has_binding_curse
      && !armor_two.priority.has_binding_curse
    {
      return 1;
    } else if armor_one.part != ArmorPart::Helmet
      && armor_one.priority.material_rank < 5
      && armor_one.priority.has_binding_curse
      && !armor_two.priority.has_binding_curse
    {
      return 1;
    }

    let (choice_by_material_rank, rank_difference) = self.compare_armors_by_material_rank(armor_one, armor_two);

    // "Какие бы чары не были на золотой броне, алмазная всегда будет лучше"
    if rank_difference >= 2 {
      return choice_by_material_rank;
    }

    let choice_by_durability = self.compare_armors_by_durability(armor_one, armor_two);
    let (choice_by_enchantments, enchantment_difference) = self.compare_armors_by_enchantments(armor_one, armor_two);

    if enchantment_difference <= 8 {
      if let Some(physics) = bot.get_physics() {
        if physics.is_in_lava() {
          if armor_one.priority.fire_protection > armor_two.priority.fire_protection {
            return 0;
          } else if armor_two.priority.fire_protection > armor_one.priority.fire_protection {
            return 1;
          }
        }

        if physics.is_in_water() {
          if armor_one.priority.aqua_affinity > armor_two.priority.aqua_affinity {
            return 0;
          } else if armor_two.priority.aqua_affinity > armor_one.priority.aqua_affinity {
            return 1;
          }
        }
      }
    }

    // Скажем, что есть железная броня на починку + защиту 4, и простая алмазная броня,
    // в данном случае естественно лучше будет взять железную, поэтому здесь такая проверка
    if (armor_one.priority.current_durability >= (armor_one.priority.max_durability / 100) * 60)
      && (armor_two.priority.current_durability >= (armor_two.priority.max_durability / 100) * 60)
      && choice_by_material_rank != 2
      && rank_difference < 2
      && enchantment_difference >= 5
    {
      return choice_by_enchantments;
    }

    if choice_by_enchantments == 2 && choice_by_durability != 2 {
      return choice_by_durability;
    }

    choice_by_enchantments
  }

  /// Метод сравнения элементов брони по рангам
  fn compare_armors_by_material_rank(&self, armor_one: &Armor, armor_two: &Armor) -> (u8, u8) {
    if armor_one.priority.material_rank != armor_two.priority.material_rank {
      let difference = if armor_one.priority.material_rank > armor_two.priority.material_rank {
        armor_one.priority.material_rank - armor_two.priority.material_rank
      } else {
        armor_two.priority.material_rank - armor_one.priority.material_rank
      };

      if armor_one.priority.material_rank > armor_two.priority.material_rank {
        return (0, difference);
      } else {
        return (1, difference);
      }
    }

    (2, 0)
  }

  /// Метод сравнения элементов брони по прочности
  fn compare_armors_by_durability(&self, armor_one: &Armor, armor_two: &Armor) -> u8 {
    if armor_one.priority.current_durability != armor_two.priority.current_durability {
      if armor_one.priority.current_durability > armor_two.priority.current_durability {
        return 0;
      } else {
        return 1;
      }
    }

    2
  }

  /// Метод сравнения элементов брони по чарам (возвращает выбор и разницу баллов)
  fn compare_armors_by_enchantments(&self, armor_one: &Armor, armor_two: &Armor) -> (u8, i32) {
    let mut points_one = 0;
    let mut points_two = 0;

    if armor_one.priority.has_mending {
      points_one += 1;
    }

    points_one += armor_one.priority.aqua_affinity;
    points_one += armor_one.priority.blast_protection;
    points_one += armor_one.priority.fire_protection;
    points_one += armor_one.priority.feather_falling;
    points_one += armor_one.priority.projectile_protection;
    points_one += armor_one.priority.unbreaking;

    if armor_one.priority.protection > 0 {
      // Эта чара является универсальной и как по мне самой лучшей для брони,
      // поэтому стоит дать дополнительный бал, чтобы бот ставил в приоретет
      // не "Fire Protection 4", а "Protection 4" и так далее
      points_one += armor_one.priority.protection + 1;
    }

    if armor_two.priority.has_mending {
      points_two += 1;
    }

    points_two += armor_two.priority.aqua_affinity;
    points_two += armor_two.priority.blast_protection;
    points_two += armor_two.priority.fire_protection;
    points_two += armor_two.priority.feather_falling;
    points_two += armor_two.priority.projectile_protection;
    points_two += armor_two.priority.unbreaking;

    if armor_two.priority.protection > 0 {
      points_two += armor_two.priority.protection + 1;
    }

    // Если на броне есть чара "проклятье несъёмности", то необходимо уменьшить баллы на 80%
    if armor_one.priority.has_binding_curse {
      points_one = (points_one / 100) * 20;
    }

    if armor_two.priority.has_binding_curse {
      points_two = (points_two / 100) * 20;
    }

    if points_one != points_two {
      let difference = if points_one > points_two {
        points_one - points_two
      } else {
        points_two - points_one
      };

      if points_one > points_two {
        return (0, difference);
      } else {
        return (1, difference);
      }
    }

    (2, 0)
  }
}

impl SalarixiPlugin for InstantArmorEquipPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index).await;
        sleep!(200);
      }
    });

    PLUGINS.push_task(&index, PluginName::InstantArmorEquip, task);
  }
}
