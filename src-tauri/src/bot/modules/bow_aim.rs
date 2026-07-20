use azalea::prelude::*;
use azalea::registry::builtin::ItemKind;
use azalea::Vec3;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::bot::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, EntityType};
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushetsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::other::tools::*;
use crate::{sleep, take_bot};

#[derive(PartialEq, Serialize_repr, Deserialize_repr)]
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

#[derive(Deserialize)]
pub struct BowAimOptions {
  target: Target,
  target_nickname: Option<String>,
  delay: Option<u64>,
  max_distance: Option<f64>,
  state: u8,
}

pub struct BowAimModule;

impl BowAimModule {
  pub fn new() -> Self {
    Self
  }

  fn find_bow_in_inventory(bot: &Client) -> Option<usize> {
    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if item.kind() == ItemKind::Bow {
          return Some(slot);
        }
      }
    }

    None
  }

  async fn aiming(index: u8, target: String, distance: f64) {
    let extra_task = tokio::spawn(async move {
      take_bot!(&index, async |bot| {
        if !getst(&index, State::CanLooking).await {
          return;
        }

        setmst(&index, StateName::Looking, true).await;

        loop {
          if !gettskact(&index, TaskName::BowAim).await {
            break;
          }

          if let Some(entity) = bot.find_nearest_entity(EntityType::from(target.clone()), distance) {
            let target_pos = bot.get_entity_position(entity);

            bot.look_at(Vec3::new(
              target_pos.x,
              target_pos.y + bot.get_entity_eye_height(entity),
              target_pos.z,
            ));
          }

          sleep!(50);
        }
      });
    });

    pushetsk(&index, TaskName::BowAim, extra_task).await;
  }

  async fn shoot(bot: &Client, index: &u8, target: String, distance: f64) {
    if getst(index, State::CanInteracting).await
      && !getst(index, State::IsEating).await
      && !getst(index, State::IsDrinking).await
      && !getst(index, State::IsAttacking).await
    {
      if let Some(slot) = Self::find_bow_in_inventory(bot) {
        setmst(index, StateName::Interacting, true).await;

        bot.take_item(index, slot, true).await;

        sleep!(50);

        bot.start_use_item();

        sleep!(randnum(900, 1100));

        if let Some(entity) = bot.find_nearest_entity(EntityType::from(target), distance) {
          let target_pos = bot.get_entity_position(entity);

          let distance = bot.eye_pos().distance_to(target_pos);

          bot.look_at(Vec3::new(target_pos.x, target_pos.y + distance * 0.16, target_pos.z));

          if distance > 50.0 {
            bot.jump();

            sleep!(100);

            let target_pos = bot.get_entity_position(entity);

            let distance = bot.eye_pos().distance_to(target_pos);

            bot.look_at(Vec3::new(target_pos.x, target_pos.y + distance * 0.12, target_pos.z));
          }

          sleep!(50);
        }

        bot.release_use_item();

        setmst(index, StateName::Interacting, false).await;
      }
    }
  }

  async fn start(bot: &Client, index: &u8, options: &BowAimOptions) {
    let target = if options.target == Target::Custom {
      options.target_nickname.clone()
    } else {
      Some(options.target.to_string())
    };

    let Some(target) = target else {
      return;
    };

    Self::aiming(*index, target.clone(), options.max_distance.unwrap_or(70.0)).await;

    loop {
      Self::shoot(bot, index, target.clone(), options.max_distance.unwrap_or(70.0)).await;
      sleep!(options.delay.unwrap_or(50));
    }
  }
}

impl SalarixiModule<BowAimOptions> for BowAimModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: BowAimOptions) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::BowAim).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::BowAim, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::BowAim).await {
      killtsk(&index, TaskName::BowAim).await;

      take_bot!(&index, async |bot| {
        bot.release_use_item();
      });

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;
    } else {
      return false;
    }

    true
  }
}
