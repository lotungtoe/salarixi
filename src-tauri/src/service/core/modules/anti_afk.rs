use azalea::prelude::*;
use azalea::WalkDirection;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::controllers::gettskact;
use crate::service::core::controllers::{killtsk, pushetsk, pushrtsk, setmst, StateName, TaskName};
use crate::service::core::extensions::{BotDefaultExt, BotMovementExt};
use crate::service::core::traits::SalarixiModule;
use crate::service::tools::*;
use crate::take_bot;

pub struct AntiAfkModule;

#[derive(Deserialize)]
pub struct AntiAfkOptions {
  pub mode: String,
  pub min_delay: Option<u64>,
  pub max_delay: Option<u64>,
  pub state: bool,
}

impl AntiAfkModule {
  pub fn new() -> Self {
    Self
  }

  async fn minimal(bot: &Client, options: &AntiAfkOptions) {
    let min_delay = options.min_delay.unwrap_or(1000);
    let max_delay = options.max_delay.unwrap_or(2500);

    loop {
      let original_direction = bot.direction();

      bot.set_direction(
        original_direction.0 + randnum(-50.0, 50.0) as f32,
        original_direction.1 + randnum(-12.0, 12.0) as f32,
      );

      if randchance(0.4) {
        bot.swing_arm();
      }

      sleep(Duration::from_millis(randnum(400, 800))).await;

      bot.set_direction(
        original_direction.0 + randnum(-10.0, 10.0) as f32,
        original_direction.1 + randnum(-4.5, 4.5) as f32,
      );

      sleep(Duration::from_millis(randnum(min_delay, max_delay))).await;
    }
  }

  async fn normal(bot: &Client, options: &AntiAfkOptions) {
    let min_delay = options.min_delay.unwrap_or(1000);
    let max_delay = options.max_delay.unwrap_or(2500);

    loop {
      let walk_directions = vec![
        WalkDirection::Forward,
        WalkDirection::Backward,
        WalkDirection::Left,
        WalkDirection::Right,
        WalkDirection::ForwardLeft,
        WalkDirection::ForwardRight,
        WalkDirection::BackwardLeft,
        WalkDirection::BackwardRight,
      ];

      let walk_direction = randelem(&walk_directions);

      bot.start_walking(*walk_direction).await;

      let direction = bot.direction();

      bot.set_direction(
        direction.0 + randnum(-35.0, 35.0) as f32,
        direction.1 + randnum(-20.0, 20.0) as f32,
      );

      sleep(Duration::from_millis(randnum(150, 400))).await;

      bot.stop_move().await;

      sleep(Duration::from_millis(randnum(min_delay, max_delay))).await;
    }
  }

  async fn advanced(bot: &Client, options: &AntiAfkOptions) {
    let min_delay = options.min_delay.unwrap_or(1000);
    let max_delay = options.max_delay.unwrap_or(2500);

    let username = bot.username();
    let username_clone = username.clone();

    let extra_task = tokio::spawn(async move {
      loop {
        take_bot!(&username_clone, async |bot| {
          let direction = bot.direction();

          bot.set_direction(
            direction.0 + randnum(-35.0, 35.0) as f32,
            direction.1 + randnum(-10.0, 10.0) as f32,
          );
        });

        sleep(Duration::from_millis(randnum(100, 400))).await;
      }
    });

    pushetsk(&username, TaskName::AntiAfk, extra_task).await;

    loop {
      bot.start_walking(WalkDirection::Forward).await;

      if randchance(0.4) {
        bot.swing_arm();
      }

      if randchance(0.4) {
        bot.start_crouching();
        sleep(Duration::from_millis(randnum(300, 500))).await;
        bot.stop_crouching();
      }

      if randchance(0.3) {
        bot.jump();
      }

      sleep(Duration::from_millis(randnum(min_delay, max_delay))).await;
    }
  }

  async fn start(bot: &Client, options: &AntiAfkOptions) {
    match options.mode.as_str() {
      "minimal" => Self::minimal(bot, options).await,
      "normal" => Self::normal(bot, options).await,
      "advanced" => Self::advanced(bot, options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<AntiAfkOptions> for AntiAfkModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, username: String, options: AntiAfkOptions) -> bool {
    if options.state && !gettskact(&username, TaskName::AntiAfk).await {
      let username_clone = username.clone();
      let task_handle = tokio::spawn(async move {
        take_bot!(&username_clone, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&username, TaskName::AntiAfk, task_handle).await;
    } else if !options.state && gettskact(&username, TaskName::AntiAfk).await {
      killtsk(&username, TaskName::AntiAfk).await;

      take_bot!(&username, async |bot| {
        bot.stop_move().await;
        bot.stop_crouching();
      });

      setmst(&username, StateName::Looking, false).await;
    } else {
      return false;
    }

    true
  }
}
