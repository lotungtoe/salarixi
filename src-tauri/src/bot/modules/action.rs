use std::time::Duration;

use azalea::prelude::*;
use serde::Deserialize;
use tokio::time::sleep;

use crate::bot::extensions::BotDefaultExt;
use crate::bot::systems::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::other::tools::*;
use crate::take_bot;

pub struct ActionModule;

#[derive(Deserialize)]
pub struct ActionOptions {
  pub action: String,
  pub use_sync: bool,
  pub use_impulsiveness: bool,
  pub state: bool,
}

impl ActionModule {
  async fn jumping(bot: &Client, options: &ActionOptions) {
    if options.use_impulsiveness {
      loop {
        if options.use_sync {
          sleep(Duration::from_millis(1000)).await;
        } else {
          sleep(Duration::from_millis(randnum(900, 1800))).await;
        }

        bot.jump();
      }
    } else {
      if !options.use_sync {
        sleep(Duration::from_millis(randnum(200, 2000))).await;
      }

      bot.set_jumping(true);
    }
  }

  async fn shifting(bot: &Client, options: &ActionOptions) {
    if options.use_impulsiveness {
      loop {
        if options.use_sync {
          sleep(Duration::from_millis(1000)).await;
        } else {
          sleep(Duration::from_millis(randnum(900, 1800))).await;
        }

        bot.set_crouching(true);
        sleep(Duration::from_millis(350)).await;
        bot.set_crouching(false);
      }
    } else {
      if !options.use_sync {
        sleep(Duration::from_millis(randnum(200, 2000))).await;
      }

      bot.set_crouching(true);
    }
  }

  async fn waving(bot: &Client, options: &ActionOptions) {
    if options.use_impulsiveness {
      loop {
        if options.use_sync {
          sleep(Duration::from_millis(300)).await;
        } else {
          sleep(Duration::from_millis(randnum(300, 800))).await;
        }

        bot.swing_arm();
      }
    } else {
      if !options.use_sync {
        sleep(Duration::from_millis(randnum(200, 2000))).await;
      }

      loop {
        bot.swing_arm();
        sleep(Duration::from_millis(300)).await;
      }
    }
  }

  async fn start(bot: &Client, options: &ActionOptions) {
    match options.action.as_str() {
      "jumping" => Self::jumping(bot, options).await,
      "shifting" => Self::shifting(bot, options).await,
      "waving" => Self::waving(bot, options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<ActionOptions> for ActionModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, username: String, options: ActionOptions) -> bool {
    let task_name = match options.action.as_str() {
      "jumping" => TaskName::Jumping,
      "shifting" => TaskName::Shifting,
      "waving" => TaskName::Waving,
      _ => return false,
    };

    if options.state && !gettskact(&username, task_name.clone()).await {
      let username_clone = username.clone();
      let root_task = tokio::spawn(async move {
        take_bot!(&username_clone, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&username, task_name, root_task).await;
    } else if !options.state && gettskact(&username, task_name.clone()).await {
      killtsk(&username, task_name).await;

      take_bot!(&username, async |bot| match options.action.as_str() {
        "jumping" => bot.set_jumping(false),
        "shifting" => bot.set_crouching(false),
        _ => {}
      });
    } else {
      return false;
    }

    true
  }
}
