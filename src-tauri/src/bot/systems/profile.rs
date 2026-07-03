use hashbrown::HashMap;
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::api::emit::emit_log;

pub static PROFILE_SYSTEM: Lazy<ProfileSystem> = Lazy::new(|| ProfileSystem::new());

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
}

/// Система управления профилями
pub struct ProfileSystem {
  pub map: RwLock<HashMap<String, Profile>>,
}

impl ProfileSystem {
  /// Метод создания нового менеджера профилей
  pub fn new() -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
    }
  }

  /// Метод регистрации бота
  pub async fn register(&self, username: &str, password: Option<String>, email: Option<String>) {
    self
      .map
      .write()
      .await
      .insert(username.to_string(), Profile::new(username, password, email));
  }

  /// Метод очистки хранилища всех профилей
  pub async fn clear(&self) {
    self.map.write().await.clear();
    emit_log("Профили ботов очищены", "extended");
  }

  /// Метод получения количества зарегистрированных профилей
  pub async fn len(&self) -> usize {
    self.map.read().await.len()
  }

  /// Метод получения количества подключенных ботов
  pub async fn get_connected_count(&self) -> usize {
    let mut count = 0;
    let guard = self.map.read().await;

    guard.iter().for_each(|(_, profile)| {
      if profile.status == BotStatus::Connected {
        count += 1;
      }
    });

    count
  }

  /// Метод попытки получения количества подключенных ботов
  pub fn try_get_connected_count(&self) -> i32 {
    if let Ok(guard) = self.map.try_read() {
      let mut count = 0;

      guard.iter().for_each(|(_, profile)| {
        if profile.status == BotStatus::Connected {
          count += 1;
        }
      });

      return count;
    }

    -1
  }

  /// Метод получения клонированного профиля указанного бота
  pub async fn get(&self, username: &str) -> Option<Profile> {
    let guard = self.map.read().await;
    guard.get(username).cloned()
  }

  /// Метод захвата мутабельного профиля указанного бота
  pub async fn lock_mut<F>(&self, username: &str, func: F)
  where
    F: AsyncFnOnce(&mut Profile),
  {
    let mut guard = self.map.write().await;
    if let Some(profile) = guard.get_mut(username) {
      func(profile).await;
    }
  }

  /// Метод получения всех профилей
  pub async fn get_all(&self) -> HashMap<String, Profile> {
    let mut result = HashMap::new();
    let guard = self.map.read().await;

    guard.iter().for_each(|(username, profile)| {
      result.insert(username.to_string(), profile.clone());
    });

    result
  }

  /// Метод получения всех профилей подключенных ботов
  pub async fn get_all_connected(&self) -> HashMap<String, Profile> {
    let mut result = HashMap::new();
    let guard = self.map.read().await;

    guard.iter().for_each(|(username, profile)| {
      if profile.status == BotStatus::Connected {
        result.insert(username.clone(), profile.clone());
      }
    });

    result
  }

  /// Метод попытки получения профиля указанного бота
  pub fn try_get(&self, username: &str) -> Option<Profile> {
    if let Ok(guard) = self.map.try_read() {
      if let Some(p) = guard.get(username) {
        return Some(p.clone());
      }
    }

    None
  }

  /// Метод попытки получения всех профилей подключенных ботов
  pub fn try_get_all_connected(&self) -> HashMap<String, Profile> {
    let mut result = HashMap::new();

    if let Ok(guard) = self.map.try_read() {
      guard.iter().for_each(|(username, profile)| {
        if profile.status == BotStatus::Connected {
          result.insert(username.clone(), profile.clone());
        }
      });
    }

    result
  }

  pub async fn set_status(&self, username: &str, status: BotStatus) {
    let mut profiles = self.map.write().await;
    if let Some(profile) = profiles.get_mut(username) {
      profile.status = status;
    }
  }

  pub async fn set_str(&self, username: &str, field: &str, value: &str) {
    let mut profiles = self.map.write().await;
    if let Some(profile) = profiles.get_mut(username) {
      match field {
        "password" => profile.password = Some(value.to_string()),
        "email" => profile.email = Some(value.to_string()),
        "group" => profile.group = value.to_string(),
        _ => {}
      }
    }
  }

  pub async fn set_bool(&self, username: &str, field: &str, value: bool) {
    let mut profiles = self.map.write().await;
    if let Some(profile) = profiles.get_mut(username) {
      match field {
        "registered" => profile.registered = value,
        "logined" => profile.logined = value,
        "captcha_caught" => profile.captcha_caught = value,
        _ => {}
      }
    }
  }

  pub async fn set_proxy(&self, username: &str, proxy: ProfileProxy) {
    let mut profiles = self.map.write().await;
    if let Some(profile) = profiles.get_mut(username) {
      profile.proxy = proxy;
    }
  }
}

/// Вспомогательный макрос захвата профиля указанного бота
#[macro_export]
macro_rules! take_profile {
  ($username:expr, $func:expr) => {
    crate::bot::systems::PROFILE_SYSTEM.lock_mut($username, $func).await;
  };
}
