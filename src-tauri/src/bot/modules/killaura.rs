use azalea::ecs::entity::Entity;
use azalea::prelude::*;
use azalea::registry::builtin::ItemKind;
use azalea::SprintDirection;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt, BotRotationExt, EntityType};
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushetsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::other::tools::*;
use crate::{sleep, take_bot};

struct Weapon {
  slot: Option<usize>,
  priority: u8,
}

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Behavior {
  Moderate = 0,
  Aggressive = 1,
}

#[derive(PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Settings {
  Adaptive = 0,
  Manual = 1,
}

#[derive(Clone, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Target {
  Player = 0,
  Monster = 1,
  Animal = 2,
  Vehicle = 3,
  Any = 4,
  Custom = 5,
}

impl Target {
  pub fn to_string(&self) -> String {
    let str = match self {
      Self::Player => "player",
      Self::Monster => "monster",
      Self::Animal => "animal",
      Self::Vehicle => "vehicle",
      Self::Any => "any",
      Self::Custom => "custom",
    };

    str.to_string()
  }
}

#[derive(PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum WeaponType {
  Sword = 0,
  Axe = 1,
}

#[derive(Deserialize)]
pub struct KillauraOptions {
  behavior: Behavior,
  settings: Settings,
  target: Target,
  target_nickname: Option<String>,
  weapon: WeaponType,
  weapon_slot: Option<u8>,
  attack_distance: Option<f64>,
  delay: Option<u64>,
  chase_distance: Option<f64>,
  min_distance_to_target: Option<f64>,
  use_auto_weapon: bool,
  use_dodging: bool,
  use_chase: bool,
  use_critical: bool,
  state: u8,
}

struct KillauraConfig {
  target: Target,
  weapon_slot: Option<u8>,
  attack_distance: f64,
  delay: u64,
  chase_distance: f64,
  min_distance_to_target: f64,
}

pub struct KillauraModule;

impl KillauraModule {
  /// Метод создания модуля
  pub fn new() -> Self {
    Self
  }

  /// Метод создания конфига
  fn create_config(options: &KillauraOptions) -> KillauraConfig {
    if options.settings == Settings::Adaptive {
      KillauraConfig {
        target: options.target.clone(),
        weapon_slot: options.weapon_slot,
        attack_distance: 3.1,
        delay: 500,
        chase_distance: options.chase_distance.unwrap_or(10.0),
        min_distance_to_target: options.min_distance_to_target.unwrap_or(3.0),
      }
    } else {
      KillauraConfig {
        target: options.target.clone(),
        weapon_slot: options.weapon_slot,
        attack_distance: options.attack_distance.unwrap_or(3.1),
        delay: options.delay.unwrap_or(500),
        chase_distance: options.chase_distance.unwrap_or(10.0),
        min_distance_to_target: options.min_distance_to_target.unwrap_or(3.0),
      }
    }
  }

  /// Метод поиска ближайшей сущности
  fn find_nearest_entity(
    bot: &Client,
    target: Target,
    target_nickname: Option<String>,
    max_distance: f64,
  ) -> Option<Entity> {
    let target = if target == Target::Custom {
      target_nickname
    } else {
      Some(target.to_string())
    };

    let Some(target) = target else {
      return None;
    };

    bot.find_nearest_entity(EntityType::from(target), max_distance)
  }

  /// Метод взятия оружия в руку
  async fn take_weapon(bot: &Client, index: &u8, weapon_type: &WeaponType) {
    let mut weapons = vec![];

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if !item.is_empty() {
          match weapon_type {
            WeaponType::Sword => match item.kind() {
              ItemKind::WoodenSword => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 0,
                });
              }
              ItemKind::GoldenSword => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 1,
                });
              }
              ItemKind::StoneSword => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 2,
                });
              }
              ItemKind::CopperSword => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 3,
                });
              }
              ItemKind::IronSword => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 4,
                });
              }
              ItemKind::DiamondSword => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 5,
                });
              }
              ItemKind::NetheriteSword => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 6,
                });
              }
              _ => {}
            },
            WeaponType::Axe => match item.kind() {
              ItemKind::WoodenAxe => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 0,
                });
              }
              ItemKind::GoldenAxe => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 1,
                });
              }
              ItemKind::StoneAxe => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 2,
                });
              }
              ItemKind::CopperAxe => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 3,
                });
              }
              ItemKind::IronAxe => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 4,
                });
              }
              ItemKind::DiamondAxe => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 5,
                });
              }
              ItemKind::NetheriteAxe => {
                weapons.push(Weapon {
                  slot: Some(slot),
                  priority: 6,
                });
              }
              _ => {}
            },
          }
        }
      }
    }

    let mut best_weapon = Weapon {
      slot: None,
      priority: 0,
    };

    for w in weapons {
      if w.priority > best_weapon.priority {
        best_weapon = w;
      }
    }

    if let Some(slot) = best_weapon.slot {
      bot.take_item(index, slot, true).await;
    }
  }

  /// Метод включения доджинга
  async fn start_dodging(index: u8) {
    let extra_task = tokio::spawn(async move {
      loop {
        take_bot!(&index, async |bot| {
          bot.start_crouching();
          sleep!(randnum(100, 800));
          bot.stop_crouching();
        });

        sleep!(randnum(400, 1500));
      }
    });

    pushetsk(&index, TaskName::Killaura, extra_task).await;
  }

  /// Метод включения преследования
  async fn start_chase(
    index: u8,
    target: Target,
    target_nickname: Option<String>,
    distance: f64,
    min_distance_to_target: f64,
  ) {
    let extra_task = tokio::spawn(async move {
      loop {
        take_bot!(&index, async |bot| {
          if let Some(entity) = Self::find_nearest_entity(&bot, target.clone(), target_nickname.clone(), distance) {
            let eye_pos = bot.eye_pos();

            if eye_pos.distance_to(bot.get_entity_position(entity)) > min_distance_to_target {
              bot.start_jumping();
              bot.start_sprinting(&index, SprintDirection::Forward).await;

              if getst(&index, State::CanLooking).await {
                setmst(&index, StateName::Looking, true).await;
                sleep!(50);
                bot.look_at_entity(entity, false);
                sleep!(50);
                setmst(&index, StateName::Looking, false).await;
              }
            } else {
              bot.stop_jumping();

              if getst(&index, State::IsSprinting).await {
                bot.stop_move(&index).await;
              }
            }
          } else {
            bot.stop_jumping();

            if getst(&index, State::IsSprinting).await {
              bot.stop_move(&index).await;
            }
          }
        });

        sleep!(50);
      }
    });

    pushetsk(&index, TaskName::Killaura, extra_task).await;
  }

  /// Метод включения автоматической наводки
  async fn start_aiming(index: u8, target: Target, target_nickname: Option<String>, distance: f64) {
    let extra_task = tokio::spawn(async move {
      loop {
        if !getst(&index, State::CanLooking).await {
          sleep!(200);
          continue;
        }

        let mut nearest_entity_exist = false;

        take_bot!(&index, async |bot| {
          if let Some(entity) = Self::find_nearest_entity(&bot, target.clone(), target_nickname.clone(), distance) {
            nearest_entity_exist = true;

            setmst(&index, StateName::Looking, true).await;
            sleep!(randnum(50, 100));
            bot.look_at_entity(entity, false);
            sleep!(randnum(50, 100));
            setmst(&index, StateName::Looking, false).await;
          }
        });

        if !nearest_entity_exist {
          sleep!(200);
        }
      }
    });

    pushetsk(&index, TaskName::Killaura, extra_task).await;
  }

  /// Метод атаки указанной сущности
  async fn attack(bot: &Client, index: &u8, options: &KillauraOptions, config: &KillauraConfig) {
    let Some(entity) = Self::find_nearest_entity(
      bot,
      config.target.clone(),
      options.target_nickname.clone(),
      config.attack_distance,
    ) else {
      return;
    };

    setst(index, State::CanDrinking, false).await;
    setst(index, State::CanInteracting, false).await;
    setst(index, State::CanEating, false).await;
    setmst(index, StateName::Attacking, true).await;

    if options.use_auto_weapon {
      Self::take_weapon(bot, index, &options.weapon).await;
    } else {
      if let Some(slot) = config.weapon_slot {
        if slot <= 8 {
          bot.set_selected_hotbar_slot(slot);
        } else {
          Self::take_weapon(bot, index, &options.weapon).await;
        }
      } else {
        Self::take_weapon(bot, &index, &options.weapon).await;
      }
    }

    if options.use_critical {
      bot.jump();

      sleep!(randnum(500, 600));

      let bot_pos = bot.feet_pos();
      let entity_pos = bot.get_entity_position(entity);
      let distance = bot_pos.distance_to(entity_pos);

      if distance <= config.attack_distance {
        bot.attack(entity);
        return;
      }
    } else {
      bot.attack(entity);
    }

    setmst(index, StateName::Attacking, false).await;
    setst(index, State::CanDrinking, true).await;
    setst(index, State::CanInteracting, true).await;
    setst(index, State::CanEating, true).await;
  }

  /// Метод запуска умеренной киллауры
  async fn moderate_killaura(bot: &Client, index: u8, options: &KillauraOptions) {
    let config = Self::create_config(options);

    Self::start_aiming(
      index,
      config.target.clone(),
      options.target_nickname.clone(),
      config.attack_distance,
    )
    .await;

    if options.use_chase {
      Self::start_chase(
        index,
        config.target.clone(),
        options.target_nickname.clone(),
        config.chase_distance,
        config.min_distance_to_target,
      )
      .await;
    }

    if options.use_dodging {
      Self::start_dodging(index).await;
    }

    loop {
      Self::attack(bot, &index, options, &config).await;
      sleep!(config.delay);
    }
  }

  /// Метод запуска агрессивной киллауры
  async fn aggressive_killaura(bot: &Client, index: u8, options: &KillauraOptions) {
    let config = Self::create_config(options);

    if options.use_chase {
      Self::start_chase(
        index,
        config.target.clone(),
        options.target_nickname.clone(),
        config.chase_distance,
        config.min_distance_to_target,
      )
      .await;
    }

    if options.use_dodging {
      Self::start_dodging(index).await;
    }

    loop {
      Self::attack(bot, &index, options, &config).await;
      sleep!(config.delay);
    }
  }

  /// Метод запуска киллауры
  async fn start(bot: &Client, index: u8, options: &KillauraOptions) {
    match options.behavior {
      Behavior::Moderate => Self::moderate_killaura(bot, index, &options).await,
      Behavior::Aggressive => Self::aggressive_killaura(bot, index, &options).await,
    }
  }
}

impl SalarixiModule<KillauraOptions> for KillauraModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: KillauraOptions) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Killaura).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, index, &options).await);
      });

      pushrtsk(&index, TaskName::Killaura, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Killaura).await {
      killtsk(&index, TaskName::Killaura).await;

      take_bot!(&index, async |bot| {
        bot.stop_move(&index).await;
        bot.stop_crouching();
        bot.stop_jumping();
      });

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Attacking, false).await;
    } else {
      return false;
    }

    true
  }
}
