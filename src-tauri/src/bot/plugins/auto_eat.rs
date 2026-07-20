use azalea::entity::ActiveEffects;
use azalea::inventory::ItemStack;
use azalea::registry::builtin::{ItemKind, MobEffect};

use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::systems::tasks::{gettskact, TaskName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::other::tools::randchance;
use crate::{sleep, take_bot};

#[derive(Clone, Copy)]
struct Food {
  slot: usize,
  priority: u8,
}

pub struct AutoEatPlugin;

impl AutoEatPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8) {
    let mut satiety = 20;
    let mut health = 20.0;
    let mut active_effects = None;

    take_bot!(index, async |bot| {
      satiety = bot.get_satiety();
      health = bot.get_health();
      active_effects = bot.get_component::<ActiveEffects>();
    });

    if !self.need_to_eat(satiety, health, active_effects) {
      return;
    }

    let food_list = self.find_food_in_inventory(index).await;
    let Some(best_food) = self.get_best_food(satiety, health, &food_list).await else {
      return;
    };

    if !getst(index, State::CanEating).await {
      return;
    }

    let mut should_eat = true;

    if gettskact(index, TaskName::Killaura).await || gettskact(index, TaskName::BowAim).await {
      should_eat = !(randchance(satiety as f64 / 20.0) && randchance(health as f64 / 20.0));
    }

    if should_eat {
      self.start_eating(index, best_food.slot).await;
    } else {
      sleep!(1800);
    }
  }

  /// Метод проверки, нужно ли есть боту (опирается на здоровье, сытость, активные эффекты)
  fn need_to_eat(&self, satiety: u32, health: f32, active_effects: Option<ActiveEffects>) -> bool {
    if satiety >= 20 && health >= 15.0 {
      return false;
    }

    if let Some(effects) = active_effects {
      for (effect, data) in &effects.0 {
        match effect {
          MobEffect::Regeneration => {}
          _ => continue,
        }

        // Думаю, здесь бессмысленно проверять время эффекта,
        // всё равно следующий тик плагина вновь проверит активные
        // эффекты и на их основе сделает корректный выбор
        if (data.amplifier >= 1 && health >= 6.0) || (data.amplifier == 0 && health >= 12.0) {
          return false;
        }
      }

      for (effect, data) in &effects.0 {
        match effect {
          MobEffect::Saturation => {}
          _ => continue,
        }

        if (data.amplifier >= 1 && satiety >= 4) || (data.amplifier == 0 && satiety >= 10) {
          return false;
        }
      }
    }

    true
  }

  /// Метод отправки пакета о кушание еды
  async fn start_eating(&self, index: &u8, slot: usize) {
    setst(index, State::CanDrinking, false).await;
    setst(index, State::CanInteracting, false).await;
    setmst(index, StateName::Eating, true).await;

    take_bot!(index, async |bot| {
      bot.freeze_move(index).await;
      bot.take_item(index, slot, true).await;
      bot.start_use_item();
    });

    sleep!(1800);

    take_bot!(index, async |bot| {
      bot.unfreeze_move(index).await;
    });

    setmst(index, StateName::Eating, false).await;
    setst(index, State::CanDrinking, true).await;
    setst(index, State::CanInteracting, true).await;
  }

  /// Метод получения лучшей еды из списка
  async fn get_best_food(&self, satiety: u32, health: f32, food_list: &Vec<Food>) -> Option<Food> {
    let mut best_food = None;

    if food_list.len() > 1 {
      if satiety == 20 && health < 12.0 {
        for food in food_list {
          if food.priority == 3 {
            best_food = Some(*food);
            break;
          }
        }
      }

      if best_food.is_some() {
        return best_food;
      }

      let desired_health_priority;
      let desired_satiety_priority;

      if health < 20.0 && health >= 18.0 {
        desired_health_priority = 0;
      } else if health < 18.0 && health >= 15.0 {
        desired_health_priority = 1;
      } else if health < 15.0 && health >= 12.0 {
        desired_health_priority = 2;
      } else {
        desired_health_priority = 3;
      }

      if satiety < 20 && satiety >= 18 {
        desired_satiety_priority = 0;
      } else if satiety < 18 && satiety >= 15 {
        desired_satiety_priority = 1;
      } else if satiety < 15 && satiety >= 12 {
        desired_satiety_priority = 2;
      } else {
        desired_satiety_priority = 3;
      }

      let mut current_priority;

      if desired_health_priority > desired_satiety_priority {
        current_priority = desired_health_priority;
      } else {
        current_priority = (desired_health_priority + desired_satiety_priority) / 2;
      }

      for attempt in 0..3 {
        for food in food_list {
          if food.priority == current_priority {
            best_food = Some(*food);
            break;
          }
        }

        if attempt > 0 {
          if current_priority + 1 <= 3 {
            current_priority += 1;
          } else {
            current_priority -= 1;
          }
        }
      }
    } else {
      best_food = food_list.get(0).copied();
    }

    best_food
  }

  /// Метод поиска еды в инвентаре
  async fn find_food_in_inventory(&self, index: &u8) -> Vec<Food> {
    let mut food_list = vec![];

    take_bot!(index, async |bot| {
      if let Some(menu) = bot.get_inventory_menu() {
        menu.slots().iter().enumerate().for_each(|(slot, item)| {
          if let Some(food) = self.is_food(slot, item) {
            food_list.push(food);
          }
        });
      }
    });

    food_list
  }

  /// Метод проверки предмета, является ли тот едой
  fn is_food(&self, slot: usize, item: &ItemStack) -> Option<Food> {
    return Some(match item.kind() {
      ItemKind::GoldenApple => Food {
        slot: slot,
        priority: 3,
      },
      ItemKind::EnchantedGoldenApple => Food {
        slot: slot,
        priority: 3,
      },

      ItemKind::GoldenCarrot => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::CookedChicken => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::CookedBeef => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::CookedCod => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::CookedMutton => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::CookedPorkchop => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::CookedRabbit => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::CookedSalmon => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::BakedPotato => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::MushroomStew => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::RabbitStew => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::BeetrootSoup => Food {
        slot: slot,
        priority: 2,
      },
      ItemKind::PumpkinPie => Food {
        slot: slot,
        priority: 2,
      },

      ItemKind::Bread => Food {
        slot: slot,
        priority: 1,
      },
      ItemKind::Chicken => Food {
        slot: slot,
        priority: 1,
      },
      ItemKind::Beef => Food {
        slot: slot,
        priority: 1,
      },
      ItemKind::Cod => Food {
        slot: slot,
        priority: 1,
      },
      ItemKind::Mutton => Food {
        slot: slot,
        priority: 1,
      },
      ItemKind::Porkchop => Food {
        slot: slot,
        priority: 1,
      },
      ItemKind::Rabbit => Food {
        slot: slot,
        priority: 1,
      },
      ItemKind::Apple => Food {
        slot: slot,
        priority: 1,
      },

      ItemKind::Salmon => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::TropicalFish => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::Potato => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::ChorusFruit => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::MelonSlice => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::SweetBerries => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::GlowBerries => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::Carrot => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::Beetroot => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::DriedKelp => Food {
        slot: slot,
        priority: 0,
      },
      ItemKind::Cookie => Food {
        slot: slot,
        priority: 0,
      },

      _ => return None,
    });
  }
}

impl SalarixiPlugin for AutoEatPlugin {
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

    PLUGINS.push_task(&index, PluginName::AutoEat, task);
  }
}
