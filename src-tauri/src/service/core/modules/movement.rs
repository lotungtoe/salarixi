use azalea::prelude::*;
use azalea::WalkDirection;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::controllers::{gettskact, killtsk, pushrtsk, TaskName};
use crate::service::core::extensions::{go_to, BotMovementExt};
use crate::service::core::traits::SalarixiModule;
use crate::service::tools::*;
use crate::take_bot;

pub struct MovementModule;

#[derive(Deserialize)]
pub struct MovementOptions {
  pub mode: String,
  pub direction: String,
  pub use_sync: bool,
  pub use_impulsiveness: bool,
  pub x: Option<i32>,
  pub z: Option<i32>,
  pub state: bool,
}

impl MovementModule {
  pub fn new() -> Self {
    Self
  }

  async fn default_move(bot: &Client, options: &MovementOptions) {
    if options.use_impulsiveness {
      if !options.use_sync {
        sleep(Duration::from_millis(randnum(500, 2000))).await;
      }

      loop {
        let direction = match options.direction.as_str() {
          "forward" => WalkDirection::Forward,
          "backward" => WalkDirection::Backward,
          "left" => WalkDirection::Left,
          "right" => WalkDirection::Right,
          "forward-left" => WalkDirection::ForwardLeft,
          "forward-right" => WalkDirection::ForwardRight,
          "backward-left" => WalkDirection::BackwardLeft,
          "backward-right" => WalkDirection::BackwardRight,
          _ => return,
        };

        bot.start_walking(direction).await;

        if options.use_sync {
          sleep(Duration::from_millis(1200)).await;
        } else {
          sleep(Duration::from_millis(randnum(800, 1800))).await;
        }

        bot.stop_move().await;
      }
    } else {
      if !options.use_sync {
        sleep(Duration::from_millis(randnum(500, 2000))).await;
      }

      let direction = match options.direction.as_str() {
        "forward" => WalkDirection::Forward,
        "backward" => WalkDirection::Backward,
        "left" => WalkDirection::Left,
        "right" => WalkDirection::Right,
        "forward-left" => WalkDirection::ForwardLeft,
        "forward-right" => WalkDirection::ForwardRight,
        "backward-left" => WalkDirection::BackwardLeft,
        "backward-right" => WalkDirection::BackwardRight,
        _ => return,
      };

      bot.start_walking(direction).await;
    }
  }

  async fn pathfinder_move(bot: &Client, options: &MovementOptions) {
    if let Some(x) = options.x {
      if let Some(z) = options.z {
        if !options.use_sync {
          sleep(Duration::from_millis(randnum(500, 2000))).await;
        }

        go_to(bot.username(), x, z);
      }
    }
  }

  async fn start(bot: &Client, options: &MovementOptions) {
    match options.mode.as_str() {
      "default" => Self::default_move(bot, &options).await,
      "pathfinder" => Self::pathfinder_move(bot, &options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<MovementOptions> for MovementModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, username: String, options: MovementOptions) -> bool {
    if options.state && !gettskact(&username, TaskName::Movement).await {
      let username_clone = username.clone();
      let task_handle = tokio::spawn(async move {
        take_bot!(&username_clone, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&username, TaskName::Movement, task_handle).await;
    } else if !options.state && gettskact(&username, TaskName::Movement).await {
      killtsk(&username, TaskName::Movement).await;

      take_bot!(&username, async |bot| {
        bot.stop_pathfinding();
        bot.stop_move().await;
      });
    } else {
      return false;
    }

    true
  }
}
