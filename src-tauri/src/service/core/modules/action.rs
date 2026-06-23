use azalea::prelude::*;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::controllers::{kill_task, push_root_task, TaskName, TASKS};
use crate::service::core::extensions::BotDefaultExt;
use crate::service::core::traits::SalarixiModule;
use crate::service::generators::*;
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
          sleep(Duration::from_millis(randuint(900, 1800))).await;
        }

        bot.jump();
      }
    } else {
      if !options.use_sync {
        sleep(Duration::from_millis(randuint(200, 2000))).await;
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
          sleep(Duration::from_millis(randuint(900, 1800))).await;
        }

        bot.set_crouching(true);
        sleep(Duration::from_millis(350)).await;
        bot.set_crouching(false);
      }
    } else {
      if !options.use_sync {
        sleep(Duration::from_millis(randuint(200, 2000))).await;
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
          sleep(Duration::from_millis(randuint(300, 800))).await;
        }

        bot.swing_arm();
      }
    } else {
      if !options.use_sync {
        sleep(Duration::from_millis(randuint(200, 2000))).await;
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

    if options.state && !TASKS.get_task_activity(&username, task_name.clone()) {
      let username_clone = username.clone();
      let root_task = tokio::spawn(async move {
        take_bot!(&username_clone, async |bot| Self::start(bot, &options).await);
      });

      push_root_task(&username, task_name, root_task);
    } else if !options.state && TASKS.get_task_activity(&username, task_name.clone()) {
      kill_task(&username, task_name);

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
