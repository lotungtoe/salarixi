use azalea::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

use crate::api::emit::emit_log;
use crate::bot::systems::PROFILE_SYSTEM;
use crate::launch::process::current_options;
use crate::other::tools::*;

const DEFAULT_AUTH_TEMPLATE: &str = "$cmd $pass";

/// Функция default-авторизации бота
pub async fn default_authorize(bot: &Client) {
  let username = bot.username();

  let Some(opts) = current_options() else {
    return;
  };

  let Some(profile) = PROFILE_SYSTEM.get(&username).await else {
    return;
  };

  let mut min_delay = 2000;
  let mut max_delay = 4000;

  let mut cmd = "";
  let mut action = "";
  let mut template = DEFAULT_AUTH_TEMPLATE;

  if !profile.registered {
    if opts.basic.use_auto_register && opts.basic.register_mode == "default" {
      cmd = opts.basic.register_command.trim();
      template = opts.basic.register_template.trim();
      min_delay = opts.basic.register_min_delay;
      max_delay = opts.basic.register_max_delay;

      PROFILE_SYSTEM.set_bool(&username, "registered", true);

      action = "зарегистрировался";

      if !opts.basic.use_double_auth {
        PROFILE_SYSTEM.set_bool(&username, "logined", true);
      }
    }
  } else if !profile.logined {
    if opts.basic.use_auto_login && opts.basic.login_mode == "default" {
      cmd = opts.basic.login_command.trim();
      template = opts.basic.login_template.trim();
      min_delay = opts.basic.login_min_delay;
      max_delay = opts.basic.login_max_delay;

      action = "залогинился";

      PROFILE_SYSTEM.set_bool(&username, "logined", true);
    }
  }

  if !cmd.is_empty() {
    sleep(Duration::from_millis(randnum(min_delay, max_delay))).await;

    let msg = template
      .replace("$cmd", &cmd)
      .replace("$pass", &profile.password.unwrap_or(String::new()))
      .replace("$email", &profile.email.unwrap_or(String::new()));

    bot.chat(&msg);

    emit_log(format!("Бот {} {}: {}", &username, action, &msg), "info");
  }
}

/// Функция trigger-авторизации бота
pub async fn trigger_authorize(bot: &Client, message: String) {
  let username = bot.username();

  let Some(opts) = current_options() else {
    return;
  };

  let Some(profile) = PROFILE_SYSTEM.get(&username).await else {
    return;
  };

  let pat = if !profile.registered {
    opts.basic.register_trigger
  } else {
    opts.basic.login_trigger
  };

  if !message.to_lowercase().contains(&pat) {
    return;
  }

  let mut cmd = "";
  let mut action = "";
  let mut template = DEFAULT_AUTH_TEMPLATE;

  if !profile.registered {
    if opts.basic.use_auto_register && opts.basic.register_mode == "trigger" {
      cmd = opts.basic.register_command.trim();
      template = opts.basic.register_template.trim();

      action = "зарегистрировался";

      PROFILE_SYSTEM.set_bool(&username, "registered", true);

      if !opts.basic.use_double_auth {
        PROFILE_SYSTEM.set_bool(&username, "logined", true);
      }
    }
  } else if !profile.logined && opts.basic.login_mode == "trigger" {
    if opts.basic.use_auto_login {
      cmd = opts.basic.login_command.trim();
      template = opts.basic.login_template.trim();

      action = "залогинился";

      PROFILE_SYSTEM.set_bool(&username, "logined", true);
    }
  }

  if !cmd.is_empty() {
    let cmd = template
      .replace("$cmd", &cmd)
      .replace("$pass", &profile.password.unwrap_or(String::new()))
      .replace("$email", &profile.email.unwrap_or(String::new()));

    bot.chat(&cmd);

    emit_log(format!("Бот {} {}: {}", &username, action, &cmd), "info");
  }
}
