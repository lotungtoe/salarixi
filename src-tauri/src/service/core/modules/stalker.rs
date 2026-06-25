use azalea::prelude::*;
use azalea::SprintDirection;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::extensions::{go_to, BotDefaultExt, BotMovementExt, EntityType};
use crate::service::core::systems::{getst, gettskact, killtsk, pushrtsk, setmst, State, StateName, TaskName};
use crate::service::core::traits::SalarixiModule;
use crate::take_bot;

pub struct StalkerModule;

#[derive(Deserialize)]
pub struct StalkerOptions {
  pub mode: String,
  pub target_nickname: Option<String>,
  pub min_distance: Option<f64>,
  pub max_distance: Option<f64>,
  pub state: bool,
}

impl StalkerModule {
  pub fn new() -> Self {
    Self
  }

  async fn default_stalking(bot: &Client, options: &StalkerOptions) {
    let username = bot.username();

    loop {
      let Some(target_nickname) = options.target_nickname.clone() else {
        return;
      };

      if let Some(target) = bot.find_nearest_entity(
        EntityType::Custom(target_nickname),
        options.max_distance.unwrap_or(100.0),
      ) {
        let target_pos = bot.get_entity_position(target);
        let min_distance = options.min_distance.unwrap_or(6.0);

        if bot.feet_pos().distance_to(target_pos) > min_distance {
          bot.look_at(target_pos);
          bot.start_jumping();
          bot.start_sprinting(SprintDirection::Forward).await;
        } else {
          bot.stop_jumping();

          if getst(&username, State::IsSprinting).await {
            bot.stop_move().await;
          }
        }
      } else {
        bot.stop_jumping();

        if getst(&username, State::IsSprinting).await {
          bot.stop_move().await;
        }
      }

      sleep(Duration::from_millis(50)).await;
    }
  }

  async fn advanced_stalking(bot: &Client, options: &StalkerOptions) {
    let Some(target_nickname) = options.target_nickname.clone() else {
      return;
    };

    loop {
      if let Some(target) = bot.find_nearest_entity(
        EntityType::Custom(target_nickname.clone()),
        options.max_distance.unwrap_or(100.0),
      ) {
        let target_pos = bot.get_entity_position(target);
        let min_distance = options.min_distance.unwrap_or(6.0);

        if bot.feet_pos().distance_to(target_pos) > min_distance {
          if bot.is_goto_target_reached() {
            go_to(
              bot.username(),
              (target_pos.x - min_distance) as i32,
              (target_pos.z - min_distance) as i32,
            );
          }
        }
      }

      sleep(Duration::from_millis(200)).await;
    }
  }

  async fn start(bot: &Client, options: &StalkerOptions) {
    let username = bot.username();

    setmst(&username, StateName::Looking, true).await;

    match options.mode.as_str() {
      "default" => Self::default_stalking(bot, options).await,
      "advanced" => Self::advanced_stalking(bot, options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<StalkerOptions> for StalkerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, username: String, options: StalkerOptions) -> bool {
    if options.state && !gettskact(&username, TaskName::Stalker).await {
      let username_clone = username.clone();
      let task_handle = tokio::spawn(async move {
        take_bot!(&username_clone, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&username, TaskName::Stalker, task_handle).await;
    } else if !options.state && gettskact(&username, TaskName::Stalker).await {
      killtsk(&username, TaskName::Stalker).await;

      take_bot!(&username, async |bot| {
        bot.stop_move().await;
        bot.stop_jumping();
      });

      setmst(&username, StateName::Looking, false).await;
    } else {
      return false;
    }

    true
  }
}
