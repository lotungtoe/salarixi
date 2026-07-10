use azalea::prelude::*;
use azalea::SprintDirection;
use serde::Deserialize;

use crate::bot::extensions::{go_to, BotDefaultExt, BotMovementExt, EntityType};
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::sleep;
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

  async fn default_stalking(bot: &Client, index: &u8, options: &StalkerOptions) {
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
          bot.start_sprinting(index, SprintDirection::Forward).await;
        } else {
          bot.stop_jumping();

          if getst(index, State::IsSprinting).await {
            bot.stop_move(index).await;
          }
        }
      } else {
        bot.stop_jumping();

        if getst(index, State::IsSprinting).await {
          bot.stop_move(index).await;
        }
      }

      sleep!(50);
    }
  }

  async fn advanced_stalking(bot: &Client, index: u8, options: &StalkerOptions) {
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
              index,
              (target_pos.x - min_distance) as i32,
              (target_pos.z - min_distance) as i32,
            );
          }
        }
      }

      sleep!(200);
    }
  }

  async fn start(bot: &Client, index: &u8, options: &StalkerOptions) {
    setmst(index, StateName::Looking, true).await;

    match options.mode.as_str() {
      "default" => Self::default_stalking(bot, &index, options).await,
      "advanced" => Self::advanced_stalking(bot, *index, options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<StalkerOptions> for StalkerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: StalkerOptions) -> bool {
    if options.state && !gettskact(&index, TaskName::Stalker).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::Stalker, task_handle).await;
    } else if !options.state && gettskact(&index, TaskName::Stalker).await {
      killtsk(&index, TaskName::Stalker).await;

      take_bot!(&index, async |bot| {
        bot.stop_move(&index).await;
        bot.stop_jumping();
      });

      setmst(&index, StateName::Looking, false).await;
    } else {
      return false;
    }

    true
  }
}
