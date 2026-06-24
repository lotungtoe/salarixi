use azalea::inventory::ItemStack;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::controllers::{get_state, set_mutual_states, set_state, State, StateName};
use crate::service::core::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, BotMovementExt};
use crate::service::core::plugin_manager::PluginName;
use crate::service::core::traits::SalarixiPlugin;
use crate::service::core::{process_is_active, PLUGINS};
use crate::service::tools::randchance;
use crate::take_bot;

#[derive(Clone, Copy)]
struct Food {
  slot: usize,
  priority: u8,
}

pub struct AutoEatPlugin;

impl AutoEatPlugin {
  async fn eat(&self, bot: &Client) {
    let satiety = bot.get_satiety();
    let health = bot.get_health();
    let username = bot.username();

    if satiety < 20 || health < 15.0 {
      let food_list = self.find_food_in_inventory(bot);

      if let Some(best_food) = self.get_best_food(bot, &food_list) {
        if get_state(&username, State::CanEating)
          && !get_state(&username, State::IsDrinking)
          && !get_state(&username, State::IsInteracting)
        {
          let mut should_eat = true;

          if get_state(&username, State::IsAttacking) {
            should_eat = !(randchance(satiety as f64 / 20.0) && randchance(health as f64 / 20.0));
          }

          if should_eat {
            set_state(&username, State::CanDrinking, false);
            set_state(&username, State::CanInteracting, false);
            set_mutual_states(&username, StateName::Eating, true);

            bot.take_item(best_food.slot, false).await;
            sleep(Duration::from_millis(50)).await;
            self.start_eating(bot).await;
            sleep(Duration::from_millis(50)).await;

            set_state(&username, State::CanDrinking, true);
            set_state(&username, State::CanInteracting, true);
            set_mutual_states(&username, StateName::Eating, false);
          } else {
            sleep(Duration::from_millis(1800)).await;
          }
        }
      }
    }
  }

  /// Метод отправки пакета о кушание еды
  async fn start_eating(&self, bot: &Client) {
    bot.freeze_move();
    bot.start_use_held_item(InteractionHand::MainHand);
    sleep(Duration::from_millis(1800)).await;
    bot.unfreeze_move();
  }

  /// Метод получения лучшей еды из списка
  fn get_best_food(&self, bot: &Client, food_list: &Vec<Food>) -> Option<Food> {
    let satiety = bot.get_satiety();
    let health = bot.get_health();

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

      if best_food.is_none() {
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
      }
    } else {
      best_food = food_list.get(0).copied();
    }

    best_food
  }

  /// Метод поиска еды в инвентаре
  fn find_food_in_inventory(&self, bot: &Client) -> Vec<Food> {
    let mut food_list = vec![];

    if let Some(menu) = bot.get_inventory_menu() {
      menu.slots().iter().enumerate().for_each(|(slot, item)| {
        if let Some(food) = self.is_food(slot, item) {
          food_list.push(food);
        }
      });
    }

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

          self.eat(bot).await;
        });

        sleep(Duration::from_millis(50)).await;
      }
    });

    PLUGINS.push_root_task(&username, PluginName::AutoEatPlugin, task);
  }
}
