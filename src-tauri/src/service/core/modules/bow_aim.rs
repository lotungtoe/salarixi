use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use azalea::Vec3;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::controllers::{
  get_state, kill_task, push_extra_task, push_root_task, set_mutual_states, State, StateName, TaskName, TASKS,
};
use crate::service::core::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, EntityType};
use crate::service::core::registry::BOT_REGISTRY;
use crate::service::core::traits::SalarixiModule;
use crate::service::generators::*;

pub struct BowAimModule;

#[derive(Deserialize)]
pub struct BowAimOptions {
  pub target: String,
  pub target_nickname: Option<String>,
  pub delay: Option<u64>,
  pub max_distance: Option<f64>,
  pub state: bool,
}

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

  fn aiming(username: String, target: String, distance: f64) {
    let username_clone = username.clone();
    let extra_task = tokio::spawn(async move {
      BOT_REGISTRY
        .async_get_bot(&username_clone, async |bot| {
          if get_state(&username_clone, State::CanLooking) {
            set_mutual_states(&username_clone, StateName::Looking, true);

            loop {
              if !TASKS.get_task_activity(&username_clone, TaskName::BowAim) {
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

              sleep(Duration::from_millis(50)).await;
            }
          }
        })
        .await;
    });

    push_extra_task(&username, TaskName::BowAim, extra_task);
  }

  async fn shoot(bot: &Client, target: String, distance: f64) {
    let username = bot.name();

    if get_state(&username, State::CanInteracting)
      && !get_state(&username, State::IsEating)
      && !get_state(&username, State::IsDrinking)
      && !get_state(&username, State::IsAttacking)
    {
      if let Some(slot) = Self::find_bow_in_inventory(bot) {
        set_mutual_states(&username, StateName::Interacting, true);

        bot.take_item(slot, true).await;

        sleep(Duration::from_millis(50)).await;

        bot.start_use_held_item(InteractionHand::MainHand);

        sleep(Duration::from_millis(randuint(900, 1100))).await;

        if let Some(entity) = bot.find_nearest_entity(EntityType::from(target), distance) {
          let target_pos = bot.get_entity_position(entity);

          let distance = bot.eye_pos().distance_to(target_pos);

          bot.look_at(Vec3::new(target_pos.x, target_pos.y + distance * 0.16, target_pos.z));

          if distance > 50.0 {
            bot.jump();

            sleep(Duration::from_millis(100)).await;

            let target_pos = bot.get_entity_position(entity);

            let distance = bot.eye_pos().distance_to(target_pos);

            bot.look_at(Vec3::new(target_pos.x, target_pos.y + distance * 0.12, target_pos.z));
          }

          sleep(Duration::from_millis(50)).await;
        }

        bot.release_use_held_item();

        set_mutual_states(&username, StateName::Interacting, false);
      }
    }
  }

  async fn start(bot: &Client, options: &BowAimOptions) {
    let target = if options.target.as_str() == "custom" {
      options.target_nickname.clone()
    } else {
      Some(options.target.clone())
    };

    let Some(target) = target else {
      return;
    };

    Self::aiming(bot.name(), target.clone(), options.max_distance.unwrap_or(70.0));

    loop {
      Self::shoot(bot, target.clone(), options.max_distance.unwrap_or(70.0)).await;
      sleep(Duration::from_millis(options.delay.unwrap_or(50))).await;
    }
  }
}

impl SalarixiModule<BowAimOptions> for BowAimModule {
  fn new() -> Self {
    Self
  }

  fn switch(&self, username: String, options: BowAimOptions) -> bool {
    if options.state && !TASKS.get_task_activity(&username, TaskName::BowAim) {
      let username_clone = username.clone();
      let task_handle = tokio::spawn(async move {
        BOT_REGISTRY
          .async_get_bot(&username_clone, async |bot| Self::start(bot, &options).await)
          .await;
      });

      push_root_task(&username, TaskName::BowAim, task_handle);
    } else if !options.state && TASKS.get_task_activity(&username, TaskName::BowAim) {
      kill_task(&username, TaskName::BowAim);

      if let Some(bot) = BOT_REGISTRY.get_bot(&username) {
        bot.release_use_held_item();
      }

      set_mutual_states(&username, StateName::Looking, false);
      set_mutual_states(&username, StateName::Interacting, false);
    } else {
      return false;
    }

    true
  }
}
