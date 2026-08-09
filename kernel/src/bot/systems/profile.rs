use bytes::BufMut;
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use salarixi_extensions::buffer::BufferExt;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::server::transfer::emit_log;

pub static PROFILE_SYSTEM: Lazy<ProfileSystem> = Lazy::new(|| ProfileSystem::new());

#[derive(Clone, Serialize)]
pub struct Profile {
  pub status: BotStatus,
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

impl Profile {
  pub fn write(&self, buf: &mut bytes::BytesMut) {
    self.status.write(buf);
    self.password.write(buf);
    self.email.write(buf);
    self.proxy.write(buf);
    self.ping.write(buf);
    self.health.write(buf);
    self.registered.write(buf);
    self.logined.write(buf);
    self.captcha_caught.write(buf);
    self.group.write(buf);
  }
}

#[derive(Clone, Serialize, PartialEq)]
pub enum BotStatus {
  Waiting,
  Connected,
  Disconnected,
}

impl BotStatus {
  pub fn write(&self, buf: &mut bytes::BytesMut) {
    buf.put_u8(match self {
      Self::Waiting => 0x00,
      Self::Connected => 0x01,
      Self::Disconnected => 0x02,
    });
  }
}

#[derive(Clone, Serialize)]
pub struct ProfileProxy {
  pub ip_address: Option<String>,
  pub proxy: Option<String>,
  pub username: Option<String>,
  pub password: Option<String>,
}

impl ProfileProxy {
  pub fn write(&self, buf: &mut bytes::BytesMut) {
    self.ip_address.write(buf);
    self.proxy.write(buf);
    self.username.write(buf);
    self.password.write(buf);
  }
}

impl Profile {
  pub fn new(password: Option<String>, email: Option<String>) -> Self {
    Self {
      status: BotStatus::Waiting,
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
  pub map: RwLock<HashMap<u8, Profile>>,
}

impl ProfileSystem {
  /// Метод создания системы профилей
  pub fn new() -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
    }
  }

  /// Метод регистрации бота
  pub async fn register(&self, index: u8, password: Option<String>, email: Option<String>) {
    self.map.write().await.insert(index, Profile::new(password, email));
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
  pub async fn get(&self, index: &u8) -> Option<Profile> {
    let guard = self.map.read().await;
    guard.get(index).cloned()
  }

  /// Метод захвата мутабельного профиля указанного бота
  pub async fn lock_mut<F>(&self, index: &u8, func: F)
  where
    F: AsyncFnOnce(&mut Profile),
  {
    let mut guard = self.map.write().await;
    if let Some(profile) = guard.get_mut(index) {
      func(profile).await;
    }
  }

  /// Метод получения всех профилей
  pub async fn get_all(&self) -> HashMap<u8, Profile> {
    let mut result = HashMap::new();
    let guard = self.map.read().await;

    guard.iter().for_each(|(index, profile)| {
      result.insert(*index, profile.clone());
    });

    result
  }

  /// Метод получения всех профилей подключенных ботов
  pub async fn get_all_connected(&self) -> HashMap<u8, Profile> {
    let mut result = HashMap::new();
    let guard = self.map.read().await;

    guard.iter().for_each(|(index, profile)| {
      if profile.status == BotStatus::Connected {
        result.insert(*index, profile.clone());
      }
    });

    result
  }

  /// Метод попытки получения профиля указанного бота
  pub fn try_get(&self, index: &u8) -> Option<Profile> {
    if let Ok(guard) = self.map.try_read() {
      if let Some(p) = guard.get(index) {
        return Some(p.clone());
      }
    }

    None
  }

  /// Метод попытки получения всех профилей подключенных ботов
  pub fn try_get_all_connected(&self) -> HashMap<u8, Profile> {
    let mut result = HashMap::new();

    if let Ok(guard) = self.map.try_read() {
      guard.iter().for_each(|(index, profile)| {
        if profile.status == BotStatus::Connected {
          result.insert(*index, profile.clone());
        }
      });
    }

    result
  }

  pub async fn set_status(&self, index: &u8, status: BotStatus) {
    let mut profiles = self.map.write().await;
    if let Some(profile) = profiles.get_mut(index) {
      profile.status = status;
    }
  }

  pub async fn set_str(&self, index: &u8, field: &str, value: &str) {
    let mut profiles = self.map.write().await;
    if let Some(profile) = profiles.get_mut(index) {
      match field {
        "password" => profile.password = Some(value.to_string()),
        "email" => profile.email = Some(value.to_string()),
        "group" => profile.group = value.to_string(),
        _ => {}
      }
    }
  }

  pub async fn set_bool(&self, index: &u8, field: &str, value: bool) {
    let mut profiles = self.map.write().await;
    if let Some(profile) = profiles.get_mut(index) {
      match field {
        "registered" => profile.registered = value,
        "logined" => profile.logined = value,
        "captcha_caught" => profile.captcha_caught = value,
        _ => {}
      }
    }
  }

  pub async fn set_proxy(&self, index: &u8, proxy: ProfileProxy) {
    let mut profiles = self.map.write().await;
    if let Some(profile) = profiles.get_mut(index) {
      profile.proxy = proxy;
    }
  }
}

/// Вспомогательный макрос захвата профиля указанного бота
#[macro_export]
macro_rules! take_profile {
  ($index:expr, $func:expr) => {
    crate::bot::systems::profile::PROFILE_SYSTEM
      .lock_mut($index, $func)
      .await;
  };
}
