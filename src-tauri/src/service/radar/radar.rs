use azalea::player::GameProfileComponent;
use chrono::prelude::*;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

use crate::service::core::controllers::PROFILES;
use crate::service::core::extensions::{go_to, BotDefaultExt};
use crate::take_bot;

pub static RADAR_MANAGER: Lazy<Arc<RadarManager>> = Lazy::new(|| Arc::new(RadarManager::new()));

// Структура информации радара
#[derive(Serialize)]
pub struct RadarInfo {
  pub status: String,
  pub uuid: String,
  pub x: f64,
  pub y: f64,
  pub z: f64,
  pub observer: RadarObserver,
}

// Структура информации о наблюдателе
#[derive(Serialize)]
pub struct RadarObserver {
  pub x: f64,
  pub z: f64,
}

// Структура RadarManager
pub struct RadarManager;

impl RadarManager {
  /// Метод создания менеджера
  pub fn new() -> Self {
    Self
  }

  /// Метод поиска цели
  pub async fn find_target(&self, target: String) -> Option<RadarInfo> {
    let mut info = None;

    for username in PROFILES.get_all_connected().await.keys() {
      take_bot!(username, async |bot| {
        let Some(tab_list) = bot.get_players() else {
          return;
        };

        for uuid in tab_list.keys() {
          let Some(entity) = bot.entity_by_uuid(*uuid) else {
            continue;
          };

          let Some(profile) = bot.get_entity_component::<GameProfileComponent>(entity) else {
            continue;
          };

          if profile.0.name == target {
            let player_pos = bot.get_entity_position(entity);
            let client_pos = bot.feet_pos();

            info = Some(RadarInfo {
              status: "Найден".to_string(),
              uuid: uuid.to_string(),
              x: player_pos.x,
              y: player_pos.y,
              z: player_pos.z,
              observer: RadarObserver {
                x: client_pos.x,
                z: client_pos.z,
              },
            });
          }
        }
      });
    }

    info
  }

  /// Метод сохранения данных в указанный файл
  pub fn save_data(&self, target: String, mut path: String, filename: String, x: f64, y: f64, z: f64) {
    let date = Local::now().format("%H:%M:%S").to_string();
    let content = format!("[ {} ] {} ~ X: {}, Y: {}, Z: {}", date, target, x, y, z);

    path.push_str(&filename.replace("#t", target.as_str()));
    path.push_str(".txt");

    let mut file = OpenOptions::new().create(true).append(true).open(&path).unwrap();

    writeln!(&mut file, "{}", content).unwrap();
  }

  /// Метод преследования цели
  pub async fn follow(&self, x: i32, z: i32) {
    for username in PROFILES.get_all_connected().await.into_keys() {
      go_to(username, x, z);
    }
  }
}
