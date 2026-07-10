use azalea::prelude::*;
use regex::Regex;
use serde::Deserialize;

use crate::bot::common::get_player_uuid;
use crate::bot::extensions::BotDefaultExt;
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::bot::systems::registry::REGISTRY_SYSTEM;
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::bot::utils::radar::RADAR;
use crate::other::tools::*;
use crate::{sleep, take_bot};

pub struct ChatModule;

#[derive(Deserialize)]
pub struct ChatOptions {
  pub mode: String,
  pub message: String,
  pub use_global_chat: bool,
  pub use_text_mutation: bool,
  pub use_sync: bool,
  pub use_magic_text: bool,
  pub use_bypass: bool,
  pub use_anti_repetition: bool,
  pub min_delay: Option<u64>,
  pub max_delay: Option<u64>,
  pub state: bool,
}

impl ChatModule {
  fn create_magic_text(text: &str) -> String {
    let mut result = String::new();

    let words = text.split_whitespace();

    for word in words {
      if !result.is_empty() {
        result.push(' ');
      }

      if word.contains("http://") || word.contains("https://") || word.contains("@") {
        result.push_str(word);
        continue;
      }

      let random_chance = randnum(0.0, 1.0);

      if random_chance >= 0.9 {
        result.push_str(&word.to_lowercase());
      } else if random_chance < 0.9 && random_chance > 0.75 {
        result.push_str(&word.to_uppercase());
      } else {
        for char in word.chars() {
          let char_chance = randnum(0.0, 1.0);

          if char_chance >= 0.70 {
            let mut transformed = char.to_lowercase().to_string();

            transformed = transformed
              .replace("o", if randchance(0.5) { "0" } else { "@" })
              .replace("о", if randchance(0.5) { "0" } else { "@" })
              .replace("a", "4")
              .replace("а", "4")
              .replace("z", "3")
              .replace("з", "3")
              .replace("e", "3")
              .replace("е", "3")
              .replace("i", if randchance(0.5) { "1" } else { "!" })
              .replace("l", if randchance(0.5) { "1" } else { "!" })
              .replace("л", if randchance(0.5) { "1" } else { "!" })
              .replace("и", if randchance(0.5) { "1" } else { "!" })
              .replace("п", "5")
              .replace("p", "5")
              .replace("v", if randchance(0.5) { "8" } else { "&" })
              .replace("в", if randchance(0.5) { "8" } else { "&" })
              .replace("б", "6")
              .replace("b", "6")
              .replace("с", "$")
              .replace("s", "$");

            result.push_str(&transformed);
          } else if char_chance < 0.70 && char_chance >= 0.5 {
            result.push_str(&char.to_uppercase().to_string());
          } else {
            result.push(char);
          }
        }
      }
    }

    result
  }

  fn create_bypass_text(text: &str) -> String {
    let stray = ["numeric", "letter", "multi", "special"];
    let separators = ["|", ":", "/", "~"];

    let random_stray = randelem(&stray);
    let random_separator = randelem(&separators);

    let stray_text = match random_stray {
      &"numeric" => randstr(CharClass::Numeric, randnum(3, 7)),
      &"letter" => randstr(CharClass::Letter, randnum(3, 7)),
      &"multi" => randstr(CharClass::Multi, randnum(3, 7)),
      &"special" => randstr(CharClass::Special, randnum(3, 7)),
      _ => String::new(),
    };

    if randchance(0.3) {
      format!("{} {} {}", text, random_separator, stray_text)
    } else {
      format!("{} {} {}", stray_text, random_separator, text)
    }
  }

  async fn process_extra_tags(index: &u8, text: String) -> String {
    let mut result = text.clone();

    for tag in Regex::new(r"#radar\[\w+]").unwrap().find_iter(&text) {
      if !tag.is_empty() {
        let split_target: Vec<&str> = tag.as_str().split("[").collect();

        if let Some(target) = split_target.get(1) {
          let target_nickname = target.replace("]", "").replace(" ", "");

          if let Some(radar_info) = RADAR.find_target(target_nickname.clone()).await {
            let msg = format!(
              "{} > X: {}, Y: {}, Z: {}",
              target_nickname,
              radar_info.x.round(),
              radar_info.y.round(),
              radar_info.z.round()
            );

            result = text.replace(tag.as_str(), msg.as_str());
          } else {
            result = text.replace(tag.as_str(), "");
          }
        }
      }
    }

    for tag in Regex::new(r"#uuid\[\w+]").unwrap().find_iter(&text) {
      if !tag.is_empty() {
        let split_target: Vec<&str> = tag.as_str().split("[").collect();

        if let Some(target) = split_target.get(1) {
          let target_nickname = target.replace("]", "").replace(" ", "");

          if let Some(uuid) = get_player_uuid(target_nickname.clone()).await {
            let msg = format!("{} > UUID: {}", target_nickname, uuid);
            result = text.replace(tag.as_str(), msg.as_str());
          } else {
            result = text.replace(tag.as_str(), "");
          }
        }
      }
    }

    for tag in Regex::new(r"#pass").unwrap().find_iter(&text) {
      if !tag.is_empty() {
        if let Some(profile) = PROFILE_SYSTEM.get(index).await {
          result = text.replace(tag.as_str(), profile.password.unwrap_or("".to_string()).as_str());
        }
      }
    }

    for tag in Regex::new(r"#username").unwrap().find_iter(&text) {
      if !tag.is_empty() {
        if let Some(username) = INDEX_SYSTEM.username_by_index(index).await {
          result = text.replace(tag.as_str(), &username);
        }
      }
    }

    for tag in Regex::new(r"#health").unwrap().find_iter(&text) {
      if !tag.is_empty() {
        if let Some(profile) = PROFILE_SYSTEM.get(index).await {
          result = text.replace(tag.as_str(), profile.health.to_string().as_str());
        }
      }
    }

    for tag in Regex::new(r"#player").unwrap().find_iter(&text) {
      if !tag.is_empty() {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          if let Some(players) = bot.get_players() {
            let mut usernames = Vec::new();

            for info in players.values() {
              usernames.push(info.profile.name.clone());
            }

            result = text.replace(tag.as_str(), randelem(&usernames).as_str());
          }
        }
      }
    }

    result
  }

  async fn message(bot: &Client, index: &u8, options: &ChatOptions) {
    let mut text = options.message.clone();

    if options.use_text_mutation {
      text = Self::process_extra_tags(index, text).await;
      text = mutate_text(text);
    }

    if !options.use_sync {
      sleep!(randnum(200, 4000));
    }

    if options.use_magic_text {
      text = Self::create_magic_text(&text);
    }

    if options.use_global_chat {
      text = format!("!{}", text);
    }

    bot.chat(&text);
  }

  async fn spamming(bot: &Client, index: &u8, options: &ChatOptions) {
    let mut latest_text = String::new();

    loop {
      let mut text = options.message.clone();

      if options.use_text_mutation {
        text = Self::process_extra_tags(index, text).await;
        text = mutate_text(text);
      }

      if options.use_sync {
        sleep!(options.min_delay.unwrap_or(2000) + options.min_delay.unwrap_or(4000) / 2);
      } else {
        sleep!(randnum(
          options.min_delay.unwrap_or(2000),
          options.max_delay.unwrap_or(4000)
        ));
      }

      let mut final_text = text;

      if options.use_magic_text {
        final_text = Self::create_magic_text(&final_text);
      }

      if options.use_bypass {
        final_text = Self::create_bypass_text(&final_text);
      }

      if options.use_global_chat {
        final_text = format!("!{}", final_text);
      }

      if options.use_anti_repetition {
        if latest_text != final_text {
          bot.chat(&final_text);
          latest_text = final_text;
        }
      } else {
        bot.chat(&final_text);
      }
    }
  }

  async fn start(bot: &Client, index: &u8, options: &ChatOptions) {
    match options.mode.as_str() {
      "message" => Self::message(bot, index, options).await,
      "spamming" => Self::spamming(bot, index, options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<ChatOptions> for ChatModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: ChatOptions) -> bool {
    let is_spamming = options.mode == "spamming";

    if !is_spamming {
      tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      return true;
    }

    if options.state && !gettskact(&index, TaskName::Spamming).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::Spamming, task_handle).await;
    } else if !options.state && gettskact(&index, TaskName::Spamming).await {
      killtsk(&index, TaskName::Spamming).await;
    } else {
      return false;
    }

    true
  }
}
