use azalea::inventory::components::PotionContents;
use azalea::inventory::ItemStack;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use azalea::registry::builtin::Potion as PotionKind;
use std::time::Duration;
use tokio::time::sleep;

use crate::bot::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, BotMovementExt, BotPhysicsExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::State;
use crate::bot::systems::StateName;
use crate::bot::systems::{getst, setmst, setst};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::other::tools::*;
use crate::take_bot;

/// Структура информации и зелье
#[derive(Clone)]
struct Potion {
  kind: String,
  slot: usize,
  name: PotionKind,
}

pub struct PotionConsumerPlugin;

impl PotionConsumerPlugin {
  /// Метод обработки тика
  async fn tick(&self, bot: &Client) {
    let health = bot.get_health();

    let username = bot.username();

    if health < 20.0 {
      let potions = self.find_potion_in_inventory(bot);

      if potions.len() > 0 {
        if let Some(potion) = self.get_best_potion(bot, potions) {
          if health < 10.0 && !getst(&username, State::IsEating).await {
            setst(&username, State::CanEating, false).await;
          } else {
            setst(&username, State::CanEating, true).await;
          }

          if getst(&username, State::CanDrinking).await
            && !getst(&username, State::IsEating).await
            && !getst(&username, State::IsInteracting).await
          {
            let mut should_drink = true;

            if getst(&username, State::IsAttacking).await {
              should_drink = !randchance(health as f64 / 20.0);
            }

            if should_drink {
              setst(&username, State::CanEating, false).await;
              setst(&username, State::CanAttacking, false).await;
              setst(&username, State::CanInteracting, false).await;

              bot.take_item(potion.slot, false).await;
              sleep(Duration::from_millis(50)).await;
              self.use_potion(bot, potion.kind).await;
              sleep(Duration::from_millis(50)).await;

              setst(&username, State::CanEating, true).await;
              setst(&username, State::CanInteracting, true).await;
              setst(&username, State::CanAttacking, true).await;
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
        setmst(&username, StateName::Drinking, true).await;
        bot.freeze_move().await;
        bot.start_use_held_item(InteractionHand::MainHand);
        sleep(Duration::from_millis(2600)).await;
        bot.unfreeze_move().await;
        setmst(&username, StateName::Drinking, false).await;
      }
      "splash" => {
        let direction = bot.direction();

        bot.set_direction(direction.0 + randnum(-5.5, 5.5) as f32, randnum(87.0, 90.0) as f32);

        sleep(Duration::from_millis(randnum(400, 600))).await;
        bot.start_use_held_item(InteractionHand::MainHand);
        sleep(Duration::from_millis(randnum(300, 500))).await;

        bot.set_direction(
          direction.0 + randnum(-2.5, 2.5) as f32,
          direction.1 + randnum(-2.5, 2.5) as f32,
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

impl SalarixiPlugin for PotionConsumerPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, username: String) {
    let username_clone = username.clone();
    let task = tokio::spawn(async move {
      loop {
        take_bot!(&username_clone, async |bot| {
          if !bot.workable() {
            return;
          }

          self.tick(bot).await;
        });

        sleep(Duration::from_millis(100)).await;
      }
    });

    PLUGINS.push_task(&username, PluginName::PotionConsumer, task);
  }
}
