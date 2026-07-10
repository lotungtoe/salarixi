use azalea::ecs::entity::Entity;
use azalea::registry::builtin::ItemKind;
use azalea::{prelude::*, SprintDirection};
use serde::Deserialize;

use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt, BotRotationExt, EntityType};
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushetsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::other::tools::*;
use crate::{sleep, take_bot};

struct Weapon {
  slot: Option<usize>,
  priority: u8,
}

pub struct KillauraModule;

#[derive(Deserialize)]
pub struct KillauraOptions {
  pub behavior: String,
  pub settings: String,
  pub target: String,
  pub target_nickname: Option<String>,
  pub weapon: String,
  pub weapon_slot: Option<u8>,
  pub attack_distance: Option<f64>,
  pub delay: Option<u64>,
  pub chase_distance: Option<f64>,
  pub min_distance_to_target: Option<f64>,
  pub use_auto_weapon: bool,
  pub use_dodging: bool,
  pub use_chase: bool,
  pub use_critical: bool,
  pub state: bool,
}

#[derive(Debug)]
struct KillauraConfig {
  target: String,
  weapon_slot: Option<u8>,
  attack_distance: f64,
  delay: u64,
  chase_distance: f64,
  min_distance_to_target: f64,
}

impl KillauraModule {
  /// Метод создания модуля
  pub fn new() -> Self {
    Self
  }

  /// Метод создания конфига
  fn create_config(options: &KillauraOptions) -> KillauraConfig {
    if options.settings.as_str() == "adaptive" {
      KillauraConfig {
        target: options.target.clone(),
        weapon_slot: None,
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
    target: String,
    target_nickname: Option<String>,
    max_distance: f64,
  ) -> Option<Entity> {
    let target = if target == "custom" {
      target_nickname
    } else {
      Some(target)
    };

    let Some(target) = target else {
      return None;
    };

    bot.find_nearest_entity(EntityType::from(target), max_distance)
  }

  /// Метод взятия оружия в руку
  async fn take_weapon(bot: &Client, index: &u8, weapon: &String) {
    let mut weapons = vec![];

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if !item.is_empty() {
          match weapon.as_str() {
            "sword" => match item.kind() {
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
            "axe" => match item.kind() {
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
            _ => {}
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
    target: String,
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

              bot.look_at_entity(entity, true);
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
  async fn start_aiming(index: u8, target: String, target_nickname: Option<String>, distance: f64) {
    let extra_task = tokio::spawn(async move {
      loop {
        setmst(&index, StateName::Looking, true).await;

        take_bot!(&index, async |bot| {
          if let Some(entity) = Self::find_nearest_entity(&bot, target.clone(), target_nickname.clone(), distance) {
            bot.look_at_entity(entity, false);
          }
        });

        sleep!(randnum(50, 100));

        setmst(&index, StateName::Looking, false).await;
      }
    });

    pushetsk(&index, TaskName::Killaura, extra_task).await;
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
      if getst(&index, State::CanAttacking).await
        && !getst(&index, State::IsInteracting).await
        && !getst(&index, State::IsEating).await
        && !getst(&index, State::IsDrinking).await
      {
        if let Some(entity) = Self::find_nearest_entity(
          bot,
          options.target.clone(),
          options.target_nickname.clone(),
          config.attack_distance,
        ) {
          setmst(&index, StateName::Attacking, true).await;

          if options.use_auto_weapon {
            Self::take_weapon(bot, &index, &options.weapon).await;
          } else {
            if let Some(slot) = config.weapon_slot {
              if slot <= 8 {
                bot.set_selected_hotbar_slot(slot);
              } else {
                Self::take_weapon(bot, &index, &options.weapon).await;
              }
            } else {
              Self::take_weapon(bot, &index, &options.weapon).await;
            }
          }

          if options.use_critical {
            bot.jump();

            sleep!(randnum(500, 600));

            if let Some(e) = Self::find_nearest_entity(
              bot,
              options.target.clone(),
              options.target_nickname.clone(),
              config.attack_distance,
            ) {
              bot.attack(e);
            }
          } else {
            bot.attack(entity);
          }

          setmst(&index, StateName::Attacking, false).await;
        }
      }

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
      if getst(&index, State::CanAttacking).await
        && !getst(&index, State::IsInteracting).await
        && !getst(&index, State::IsEating).await
        && !getst(&index, State::IsDrinking).await
      {
        if let Some(entity) = Self::find_nearest_entity(
          bot,
          options.target.clone(),
          options.target_nickname.clone(),
          config.attack_distance,
        ) {
          setmst(&index, StateName::Attacking, true).await;

          if options.use_auto_weapon {
            Self::take_weapon(bot, &index, &options.weapon).await;
          } else {
            if let Some(slot) = config.weapon_slot {
              if slot <= 8 {
                bot.set_selected_hotbar_slot(slot);
              } else {
                Self::take_weapon(bot, &index, &options.weapon).await;
              }
            } else {
              Self::take_weapon(bot, &index, &options.weapon).await;
            }
          }

          if options.use_critical {
            bot.jump();
            sleep!(randnum(500, 600));

            if let Some(e) = Self::find_nearest_entity(
              bot,
              options.target.clone(),
              options.target_nickname.clone(),
              config.attack_distance,
            ) {
              bot.attack(e);
            }
          } else {
            bot.attack(entity);
          }

          setmst(&index, StateName::Attacking, false).await;
        }
      }

      sleep!(config.delay);
    }
  }

  /// Метод запуска киллауры
  async fn start(bot: &Client, index: u8, options: &KillauraOptions) {
    match options.behavior.as_str() {
      "moderate" => Self::moderate_killaura(bot, index, &options).await,
      "aggressive" => Self::aggressive_killaura(bot, index, &options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<KillauraOptions> for KillauraModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: KillauraOptions) -> bool {
    if options.state && !gettskact(&index, TaskName::Killaura).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, index, &options).await);
      });

      pushrtsk(&index, TaskName::Killaura, task_handle).await;
    } else if !options.state && gettskact(&index, TaskName::Killaura).await {
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
