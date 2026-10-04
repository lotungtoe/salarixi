use std::sync::Arc;

use azalea::entity::{LookDirection, Physics};
use azalea::inventory::components::PotionContents;
use azalea::inventory::ItemStack;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::{ItemKind, Potion as PotionKind};

use crate::bot::extensions::BotInteractExt;
use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt, BotPhysicsExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::systems::tasks::{gettskact, TaskName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::launch::options::PotionConsumerOptions;
use crate::tools::*;
use crate::{sleep, take_bot};

#[derive(Clone)]
struct Potion {
  class: PotionClass,
  slot: usize,
  name: PotionKind,
}

#[derive(Clone, PartialEq)]
enum PotionClass {
  Drinking,
  Splash,
}

pub struct PotionConsumerPlugin;

impl PotionConsumerPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8, options: &PotionConsumerOptions) {
    let mut health = 20.0;

    take_bot!(index, async |bot| {
      health = bot.get_health();
    });

    if health >= options.minimum_health_threshold {
      return;
    }

    let potions = self.find_potion_in_inventory(index, options).await;

    if potions.len() == 0 {
      sleep!(options.delay_after_no_potion as u64);
      return;
    }

    let mut current_physics = None;

    take_bot!(index, async |bot| {
      current_physics = bot.get_physics();
    });

    let Some(physics) = current_physics else {
      return;
    };

    let Some(best_potion) = self.get_best_potion(health, physics, potions) else {
      sleep!(options.delay_after_no_potion as u64);
      return;
    };

    if !getst(index, State::CanDrinking).await {
      return;
    }

    let mut should_drink = true;

    if options.reduce_drink_chance_on_combat
      && (getst(index, State::IsAttacking).await
        || gettskact(index, TaskName::Killaura).await
        || gettskact(index, TaskName::BowAim).await)
    {
      should_drink = !randchance(health as f64 / 20.0);
    }

    if !should_drink {
      return;
    }

    setst(index, State::CanEating, false).await;
    setst(index, State::CanAttacking, false).await;
    setst(index, State::CanInteracting, false).await;

    self.use_potion(index, best_potion).await;

    setst(index, State::CanEating, true).await;
    setst(index, State::CanInteracting, true).await;
    setst(index, State::CanAttacking, true).await;
  }

  /// Метод отправки пакета об использовании зелья
  async fn use_potion(&self, index: &u8, potion: Potion) {
    take_bot!(index, async |bot| {
      bot.take_item(index, potion.slot, false).await;
    });

    match potion.class {
      PotionClass::Drinking => {
        setmst(index, StateName::Drinking, true).await;

        take_bot!(index, async |bot| {
          bot.freeze_move(index).await;
          bot.start_use_item_by(InteractionHand::MainHand);
        });

        sleep!(2600);

        take_bot!(index, async |bot| {
          bot.unfreeze_move(index).await;
        });

        setmst(index, StateName::Drinking, false).await;
      }
      PotionClass::Splash => {
        let mut direction = LookDirection::default();

        take_bot!(index, async |bot| {
          direction.update(bot.direction());
          bot.set_direction(
            direction.y_rot() + randnum(-5.5, 5.5) as f32,
            randnum(87.0, 90.0) as f32,
          );
        });

        sleep!(randnum(400, 600));

        take_bot!(index, async |bot| {
          bot.start_use_item_by(InteractionHand::MainHand);
        });

        sleep!(randnum(300, 500));

        take_bot!(index, async |bot| {
          bot.set_direction(
            direction.y_rot() + randnum(-2.5, 2.5) as f32,
            direction.x_rot() + randnum(-2.5, 2.5) as f32,
          );
        });
      }
    }
  }

  /// Метод получения лучшего зелья из списка
  fn get_best_potion(&self, health: f32, physics: Physics, potions: Vec<Potion>) -> Option<Potion> {
    let is_in_lava = physics.is_in_lava();
    let is_burning = physics.remaining_fire_ticks > 0;
    let is_in_water = physics.is_in_water();
    let on_ground = physics.on_ground();
    let velocity_y = physics.velocity.y;

    let mut best_potion: Option<Potion> = None;

    for p in potions {
      if let Some(bp) = &best_potion {
        if bp.class == PotionClass::Splash {
          continue;
        }
      }

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

      if p.class == PotionClass::Splash {
        break;
      }
    }

    best_potion
  }

  /// Метод поиска зелья в инвентаре
  async fn find_potion_in_inventory(&self, index: &u8, options: &PotionConsumerOptions) -> Vec<Potion> {
    let mut potion_list = vec![];

    take_bot!(index, async |bot| {
      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      menu.slots().iter().enumerate().for_each(|(slot, item)| {
        if let Some(potion) = self.is_potion(slot, item, options) {
          potion_list.push(potion);
        }
      });
    });

    potion_list
  }

  /// Метод проверки предмета, является ли тот зельем
  fn is_potion(&self, slot: usize, item: &ItemStack, options: &PotionConsumerOptions) -> Option<Potion> {
    if options.include_drinking_potion && item.kind() == ItemKind::Potion {
      if let Some(contents) = item.get_component::<PotionContents>() {
        if let Some(potion) = contents.potion {
          return Some(Potion {
            class: PotionClass::Drinking,
            slot: slot,
            name: potion,
          });
        }
      }
    } else if options.include_splash_potion && item.kind() == ItemKind::SplashPotion {
      if let Some(contents) = item.get_component::<PotionContents>() {
        if let Some(potion) = contents.potion {
          return Some(Potion {
            class: PotionClass::Splash,
            slot: slot,
            name: potion,
          });
        }
      }
    }

    None
  }
}

impl SalarixiPlugin<PotionConsumerOptions> for PotionConsumerPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8, options: Arc<PotionConsumerOptions>) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index, &options).await;
        sleep!(options.tick_interval as u64);
      }
    });

    PLUGINS.push_task(&index, PluginName::PotionConsumer, task);
  }
}
