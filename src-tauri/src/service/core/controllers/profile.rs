use hashbrown::HashMap;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::emit::emit_log;

pub static PROFILES: Lazy<Arc<ProfileManager>> = Lazy::new(|| Arc::new(ProfileManager::new()));

/// Структура профиля бота
#[derive(Clone, Serialize)]
pub struct Profile {
  pub status: BotStatus,
  pub username: String,
  pub password: Option<String>,
  pub email: Option<String>,
  pub proxy: ProfileProxy,
  pub ping: u32,
  pub health: u32,
  pub registered: bool,
  pub logined: bool,
  pub captcha_caught: bool,
  pub group: String,
}

/// Состояние бота
#[derive(Clone, Serialize, PartialEq)]
pub enum BotStatus {
  Waiting,
  Connected,
  Disconnected,
}

/// Структура прокси бота
#[derive(Clone, Serialize)]
pub struct ProfileProxy {
  pub ip_address: Option<String>,
  pub proxy: Option<String>,
  pub username: Option<String>,
  pub password: Option<String>,
}

impl Profile {
  /// Метод создания нового профиля
  pub fn new(username: &str, password: Option<String>, email: Option<String>) -> Self {
    Self {
      status: BotStatus::Waiting,
      username: username.to_string(),
      password: password,
      email: email,
      proxy: ProfileProxy {
        ip_address: None,
        proxy: None,
        username: None,
        password: None,
      },
      ping: 0,
      health: 0,
      registered: false,
      logined: false,
      captcha_caught: false,
      group: "global".to_string(),
    }
  }

  pub fn set_status(&mut self, status: BotStatus) {
    self.status = status;
  }

  pub fn set_password(&mut self, password: &str) {
    self.password = Some(password.to_string());
  }

  pub fn set_email(&mut self, email: &str) {
    self.email = Some(email.to_string());
  }

  pub fn set_proxy(&mut self, proxy: ProfileProxy) {
    self.proxy = proxy;
  }

  pub fn set_ping(&mut self, ping: u32) {
    self.ping = ping;
  }

  pub fn set_health(&mut self, health: u32) {
    self.health = health;
  }

  pub fn set_registered(&mut self, state: bool) {
    self.registered = state;
  }

  pub fn set_logined(&mut self, state: bool) {
    self.logined = state;
  }

  pub fn set_captcha_caught(&mut self, state: bool) {
    self.captcha_caught = state;
  }

  pub fn set_group(&mut self, group: &str) {
    self.group = group.to_string();
  }
}

/// Менеджер управления профилями
pub struct ProfileManager {
  pub map: RwLock<HashMap<String, Profile>>,
}

impl ProfileManager {
  /// Метод создания нового менеджера профилей
  pub fn new() -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
    }
  }

  /// Метод регистрации бота
  pub fn register(&self, username: &str, password: Option<String>, email: Option<String>) {
    if let Ok(mut profiles) = self.map.try_write() {
      profiles.insert(username.to_string(), Profile::new(username, password, email));
    }
  }

  /// Метод очистки хранилища всех профилей
  pub async fn clear(&self) {
    self.map.write().await.clear();
    emit_log("Профили ботов очищены", "extended");
  }

  /// Метод получения количества активных ботов
  pub fn get_active_count(&self) -> i32 {
    if let Ok(profiles) = self.map.try_read() {
      let mut count = 0;

      for (_, profile) in profiles.iter() {
        if profile.status == BotStatus::Connected {
          count += 1;
        }
      }

      count
    } else {
      -1
    }
  }

  /// Метод получения профиля указанного бота
  pub fn get(&self, username: &str) -> Option<Profile> {
    if let Ok(profiles) = self.map.try_read() {
      if let Some(p) = profiles.get(username) {
        return Some(p.clone());
      }
    }

    None
  }

  /// Метод получения всех профилей
  pub fn get_all(&self) -> HashMap<String, Profile> {
    let mut result = HashMap::new();

    if let Ok(profiles) = self.map.try_read() {
      profiles.iter().for_each(|(username, profile)| {
        result.insert(username.to_string(), profile.clone());
      });
    }

    result
  }

  /// Метод получения всех профилей подключенных ботов
  pub fn get_all_connected(&self) -> HashMap<String, Profile> {
    let mut result = HashMap::new();

    if let Ok(profiles) = self.map.try_read() {
      profiles.iter().for_each(|(username, profile)| {
        if profile.status == BotStatus::Connected {
          result.insert(username.clone(), profile.clone());
        }
      });
    }

    result
  }

  pub fn set_status(&self, username: &str, status: BotStatus) {
    if let Ok(mut profiles) = self.map.try_write() {
      if let Some(p) = profiles.get_mut(username) {
        p.set_status(status);
      }
    }
  }

  pub fn set_str(&self, username: &str, field: &str, value: &str) {
    if let Ok(mut profiles) = self.map.try_write() {
      if let Some(p) = profiles.get_mut(username) {
        match field {
          "password" => p.set_password(value),
          "email" => p.set_email(value),
          "group" => p.set_group(value),
          _ => {}
        }
      }
    }
  }

  pub fn set_num(&self, username: &str, field: &str, value: u32) {
    if let Ok(mut profiles) = self.map.try_write() {
      if let Some(p) = profiles.get_mut(username) {
        match field {
          "ping" => p.set_ping(value),
          "health" => p.set_health(value),
          _ => {}
        }
      }
    }
  }

  pub fn set_bool(&self, username: &str, field: &str, value: bool) {
    if let Ok(mut profiles) = self.map.try_write() {
      if let Some(p) = profiles.get_mut(username) {
        match field {
          "registered" => p.set_registered(value),
          "logined" => p.set_logined(value),
          "captcha_caught" => p.set_captcha_caught(value),
          _ => {}
        }
      }
    }
  }

  pub fn set_proxy(&self, username: &str, proxy: ProfileProxy) {
    if let Ok(mut profiles) = self.map.try_write() {
      if let Some(p) = profiles.get_mut(username) {
        p.set_proxy(proxy);
      }
    }
  }
}
