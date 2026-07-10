use azalea::prelude::*;
use azalea::WalkDirection;
use serde::Deserialize;

use crate::bot::extensions::{go_to, BotMovementExt};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::other::tools::*;
use crate::sleep;
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

  async fn default_move(bot: &Client, index: &u8, options: &MovementOptions) {
    if options.use_impulsiveness {
      if !options.use_sync {
        sleep!(randnum(500, 2000));
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

        bot.start_walking(index, direction).await;

        if options.use_sync {
          sleep!(1200);
        } else {
          sleep!(randnum(800, 1800));
        }

        bot.stop_move(index).await;
      }
    } else {
      if !options.use_sync {
        sleep!(randnum(500, 2000));
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

      bot.start_walking(index, direction).await;
    }
  }

  async fn pathfinder_move(index: u8, options: &MovementOptions) {
    if let Some(x) = options.x {
      if let Some(z) = options.z {
        if !options.use_sync {
          sleep!(randnum(500, 2000));
        }

        go_to(index, x, z);
      }
    }
  }

  async fn start(bot: &Client, index: &u8, options: &MovementOptions) {
    match options.mode.as_str() {
      "default" => Self::default_move(bot, index, &options).await,
      "pathfinder" => Self::pathfinder_move(*index, &options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<MovementOptions> for MovementModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: MovementOptions) -> bool {
    if options.state && !gettskact(&index, TaskName::Movement).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::Movement, task_handle).await;
    } else if !options.state && gettskact(&index, TaskName::Movement).await {
      killtsk(&index, TaskName::Movement).await;

      take_bot!(&index, async |bot| {
        bot.stop_pathfinding();
        bot.stop_move(&index).await;
      });
    } else {
      return false;
    }

    true
  }
}
