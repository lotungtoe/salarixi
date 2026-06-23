use azalea::inventory::components::PotionContents;
use azalea::inventory::ItemStack;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use azalea::registry::builtin::Potion as PotionKind;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::controllers::State;
use crate::service::core::controllers::StateName;
use crate::service::core::controllers::{get_state, set_mutual_states, set_state};
use crate::service::core::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, BotMovementExt, BotPhysicsExt};
use crate::service::core::plugin_manager::PluginName;
use crate::service::core::traits::SalarixiPlugin;
use crate::service::core::{process_is_active, PLUGINS};
use crate::service::generators::*;
use crate::take_bot;

/// Структура информации и зелье
#[derive(Clone)]
struct Potion {
  kind: String,
  slot: usize,
  name: PotionKind,
}

pub struct AutoPotionPlugin;

impl AutoPotionPlugin {
  /// Метод применения зелья
  async fn drink(&self, bot: &Client) {
    let health = bot.get_health();

    let username = bot.username();

    if health < 20.0 {
      let potions = self.find_potion_in_inventory(bot);

      if potions.len() > 0 {
        if let Some(potion) = self.get_best_potion(bot, potions) {
          if health < 10.0 && !get_state(&username, State::IsEating) {
            set_state(&username, State::CanEating, false);
          } else {
            set_state(&username, State::CanEating, true);
          }

          if get_state(&username, State::CanDrinking)
            && !get_state(&username, State::IsEating)
            && !get_state(&username, State::IsInteracting)
          {
            let mut should_drink = true;

            if get_state(&username, State::IsAttacking) {
              should_drink = !randchance(health as f64 / 20.0);
            }

            if should_drink {
              set_state(&username, State::CanEating, false);
              set_state(&username, State::CanAttacking, false);
              set_state(&username, State::CanInteracting, false);

              bot.take_item(potion.slot, false).await;
              sleep(Duration::from_millis(50)).await;
              self.use_potion(bot, potion.kind).await;
              sleep(Duration::from_millis(50)).await;

              set_state(&username, State::CanEating, true);
              set_state(&username, State::CanInteracting, true);
              set_state(&username, State::CanAttacking, true);
            }
          }
        }
      }
    }
  }

  /// Метод отправки пакета об использовании зелья
  async fn use_potion(&self, bot: &Client, kind: String) {
    let username = bot.username();

    match kind.as_str() {
      "default" => {
        set_mutual_states(&username, StateName::Drinking, true);
        bot.freeze_move();
        bot.start_use_held_item(InteractionHand::MainHand);
        sleep(Duration::from_millis(2600)).await;
        bot.unfreeze_move();
        set_mutual_states(&username, StateName::Drinking, false);
      }
      "splash" => {
        let direction = bot.direction();

        bot.set_direction(direction.0 + randfloat(-5.5, 5.5) as f32, randfloat(87.0, 90.0) as f32);

        sleep(Duration::from_millis(randuint(400, 600))).await;
        bot.start_use_held_item(InteractionHand::MainHand);
        sleep(Duration::from_millis(randuint(300, 500))).await;

        bot.set_direction(
          direction.0 + randfloat(-2.5, 2.5) as f32,
          direction.1 + randfloat(-2.5, 2.5) as f32,
        );
      }
      _ => {}
    }
  }

  /// Метод получения лучшего зелья из списка
  fn get_best_potion(&self, bot: &Client, potions: Vec<Potion>) -> Option<Potion> {
    let health = bot.get_health();

    if let Some(physics) = bot.get_physics() {
      let is_in_lava = physics.is_in_lava();
      let is_burning = physics.remaining_fire_ticks > 0;
      let is_in_water = physics.is_in_water();
      let on_ground = physics.on_ground();
      let velocity_y = physics.velocity.y;

      let mut best_potion = None;

      for p in potions {
        if best_potion
          .as_ref()
          .unwrap_or(&Potion {
            kind: "deafult".to_string(),
            slot: 0,
            name: PotionKind::Awkward,
          })
          .kind
          .as_str()
          != "splash"
        {
          if best_potion.is_none() && is_in_lava || is_burning {
            match p.name {
              PotionKind::FireResistance => {
                best_potion = Some(p.clone());
              }
              PotionKind::LongFireResistance => {
                best_potion = Some(p.clone());
              }
              _ => {}
            }
          }

          if best_potion.is_none() && is_in_water {
            match p.name {
              PotionKind::WaterBreathing => {
                best_potion = Some(p.clone());
              }
              PotionKind::LongWaterBreathing => {
                best_potion = Some(p.clone());
              }
              _ => {}
            }
          }

          if best_potion.is_none() && !on_ground && velocity_y < -0.5 {
            match p.name {
              PotionKind::SlowFalling => {
                best_potion = Some(p.clone());
              }
              PotionKind::LongSlowFalling => {
                best_potion = Some(p.clone());
              }
              _ => {}
            }
          }

          if best_potion.is_none() && health <= 8.0 {
            match p.name {
              PotionKind::TurtleMaster => {
                best_potion = Some(p.clone());
              }
              PotionKind::LongTurtleMaster => {
                best_potion = Some(p.clone());
              }
              PotionKind::StrongTurtleMaster => {
                best_potion = Some(p.clone());
              }
              _ => {}
            }
          }

          if best_potion.is_none() && health <= 15.0 {
            match p.name {
              PotionKind::Regeneration => {
                best_potion = Some(p);
                break;
              }
              PotionKind::LongRegeneration => {
                best_potion = Some(p);
                break;
              }
              PotionKind::StrongRegeneration => {
                best_potion = Some(p);
                break;
              }
              PotionKind::Healing => {
                best_potion = Some(p);
                break;
              }
              PotionKind::StrongHealing => {
                best_potion = Some(p);
                break;
              }
              _ => {}
            }
          }

          if p.kind.as_str() == "splash" {
            break;
          }
        }
      }

      return best_potion;
    }

    None
  }

  /// Метод поиска зелья в инвентаре
  fn find_potion_in_inventory(&self, bot: &Client) -> Vec<Potion> {
    let mut potion_list = vec![];

    if let Some(menu) = bot.get_inventory_menu() {
      menu.slots().iter().enumerate().for_each(|(slot, item)| {
        if let Some(potion) = self.is_potion(slot, item) {
          potion_list.push(potion);
        }
      });
    }

    potion_list
  }

  /// Метод проверки предмета, является ли тот зельем
  fn is_potion(&self, slot: usize, item: &ItemStack) -> Option<Potion> {
    match item.kind() {
      ItemKind::Potion => {
        if let Some(contents) = item.get_component::<PotionContents>() {
          if let Some(potion) = contents.potion {
            return Some(Potion {
              kind: "default".to_string(),
              slot: slot,
              name: potion,
            });
          }
        }
      }
      ItemKind::SplashPotion => {
        if let Some(contents) = item.get_component::<PotionContents>() {
          if let Some(potion) = contents.potion {
            return Some(Potion {
              kind: "splash".to_string(),
              slot: slot,
              name: potion,
            });
          }
        }
      }
      _ => {}
    }

    None
  }
}

impl SalarixiPlugin for AutoPotionPlugin {
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

          self.drink(bot).await;
        });

        sleep(Duration::from_millis(50)).await;
      }
    });

    PLUGINS.push_root_task(&username, PluginName::AutoPotionPlugin, task);
  }
}
