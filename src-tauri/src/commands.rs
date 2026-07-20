use std::fs;
use std::time::Duration;

use hashbrown::HashMap;
use serde::Deserialize;

use crate::bot::functions::{disconnect_bot, reset_bot, send_message_from_bot};
use crate::bot::script::SCRIPT_EXECUTOR;
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::{Profile, PROFILE_SYSTEM};
use crate::bot::systems::registry::{RegistryEvent, REGISTRY_SYSTEM};
use crate::bot::utils::ping::{ping_server, ServerInformation};
use crate::bot::utils::radar::{RadarInfo, RADAR};
use crate::emit::{emit_log, emit_msg};
use crate::launch::options::LaunchOptions;
use crate::launch::process::{current_options, process_is_active};
use crate::launch::runner::{launch_bots_on_server, stop_bots_and_destroy_data};
use crate::other::discord_rpc::DISCORD_RPC;
use crate::other::webhook::send_webhook;

/// Команда запуска ботов
#[tauri::command]
pub async fn launch_bots(options: Vec<u8>) -> bool {
  let parsed_options: LaunchOptions = match serde_json::from_slice(&options) {
    Ok(p) => p,
    Err(_) => return false,
  };

  launch_bots_on_server(parsed_options).await
}

/// Команда остановки ботов
#[tauri::command]
pub async fn stop_bots() -> bool {
  stop_bots_and_destroy_data().await
}

/// Команда получения профилей ботов
#[tauri::command]
pub async fn get_bot_profiles() -> HashMap<String, Profile> {
  let mut profiles = HashMap::new();
  let index_map = INDEX_SYSTEM.map.read().await;
  for (index, profile) in PROFILE_SYSTEM.get_all().await {
    if let Some(username) = index_map.get(index as usize) {
      profiles.insert(username.clone(), profile);
    }
  }

  profiles
}

#[derive(Deserialize)]
struct SendMessageCommand {
  username: String,
  message: String,
}

#[derive(Deserialize)]
struct ResetBotCommand {
  username: String,
}

#[derive(Deserialize)]
struct DisconnectBotCommand {
  username: String,
}

/// Команда выполнения быстрой операции
#[tauri::command]
pub async fn quick_operation(operation: String, options: serde_json::Value) {
  match operation.as_str() {
    "send-message" => {
      let opts: SendMessageCommand = match serde_json::from_value(options) {
        Ok(o) => o,
        Err(_) => return,
      };

      send_message_from_bot(opts.username, opts.message).await;
    }
    "reset-bot" => {
      let opts: ResetBotCommand = match serde_json::from_value(options) {
        Ok(o) => o,
        Err(_) => return,
      };

      reset_bot(opts.username).await;
    }
    "disconnect-bot" => {
      let opts: DisconnectBotCommand = match serde_json::from_value(options) {
        Ok(o) => o,
        Err(_) => return,
      };

      disconnect_bot(opts.username).await;
    }
    _ => {}
  }
}

/// Команда установки группы указанному боту
#[tauri::command]
pub async fn set_group(username: String, group: String) {
  if let Some(index) = INDEX_SYSTEM.index_by_username(&username).await {
    PROFILE_SYSTEM.set_str(&index, "group", &group).await;
  }
}

/// Команда получения количества активных ботов
#[tauri::command]
pub async fn get_active_bots_count() -> i32 {
  PROFILE_SYSTEM.get_connected_count().await as i32
}

/// Команда управления ботами
#[tauri::command]
pub async fn control_bots(name: String, options: serde_json::Value, group: String) {
  if process_is_active() {
    REGISTRY_SYSTEM.send_event(RegistryEvent::ControlModules { name, options, group });
  }
}

/// Команда пингования сервера
#[tauri::command]
pub async fn get_server_info(address: String) -> ServerInformation {
  ping_server(address).await
}

/// Команда получения данных радара
#[tauri::command]
pub async fn get_radar_data(target: String) -> Option<RadarInfo> {
  RADAR.find_target(target).await
}

/// Команда сохранения данных радара
#[tauri::command]
pub async fn save_radar_data(target: String, path: String, filename: String, x: f64, y: f64, z: f64) {
  RADAR.save_data(target, path, filename, x, y, z);
}

/// Команда преследования цели радара
#[tauri::command]
pub async fn follow_radar_target(x: i32, z: i32) {
  RADAR.follow(x, z).await;
}

/// Команда получения используемой памяти
#[tauri::command]
pub async fn get_memory_usage() -> f64 {
  if let Some(usage) = memory_stats::memory_stats() {
    return (usage.physical_mem as f64 / 1024.0) / 1024.0;
  }

  0.0
}

/// Команда выполнения быстрой задачи
#[tauri::command]
pub async fn quick_task(name: String) {
  if let Some(opts) = current_options().await {
    if opts.basic.use_webhook && opts.webhook.send_actions {
      send_webhook(opts.webhook.url, format!("Быстрая задача '{}'", name));
    }
  }

  emit_log(format!("Быстрая задача '{}'", name), "extended");

  emit_msg(
    "Быстрая задача",
    format!(
      "{} ботов получили быструю задачу '{}'",
      PROFILE_SYSTEM.get_connected_count().await,
      name
    ),
  );

  REGISTRY_SYSTEM.send_event(RegistryEvent::QuickTask { name });
}

/// Команда выполнения пользовательского скрипта
#[tauri::command]
pub async fn execute_script(script: String, separately: bool) {
  if process_is_active() {
    if separately {
      for (index, _) in PROFILE_SYSTEM.get_all_connected().await {
        let Some(username) = INDEX_SYSTEM.username_by_index(&index).await else {
          continue;
        };

        let script_clone = script.clone();

        tokio::spawn(async move {
          SCRIPT_EXECUTOR.execute(username, index, script_clone);
        });
      }
    } else {
      SCRIPT_EXECUTOR.execute(String::new(), 0, script);
    }
  }
}

/// Команда остановки пользовательского скрипта
#[tauri::command]
pub async fn stop_script() {
  SCRIPT_EXECUTOR.stop();
}

/// Команда открытия URL в браузере
#[tauri::command]
pub async fn open_url(url: String) {
  let _ = open::that(url);
}

/// Команда скачивания JSON-контента
#[tauri::command]
pub async fn download_json(url: String) -> Option<serde_json::Value> {
  let client = reqwest::Client::new();

  let resp = client
    .get(url)
    .timeout(Duration::from_secs(10))
    .header("Content-Type", "application/json")
    .send()
    .await
    .ok()?;

  if resp.status() != reqwest::StatusCode::OK {
    return None;
  }

  resp.json().await.ok()
}

/// Команда скачивания TEXT-контента
#[tauri::command]
pub async fn download_text(url: String) -> Option<String> {
  let client = reqwest::Client::new();

  let resp = client.get(url).timeout(Duration::from_secs(10)).send().await.ok()?;

  if resp.status() != reqwest::StatusCode::OK {
    return None;
  }

  resp.text().await.ok()
}

/// Команда чтения небольшого текстового файла
#[tauri::command]
pub fn read_text_file(path: String) -> Vec<u8> {
  match fs::read(&path) {
    Ok(b) => b,
    Err(_) => Vec::new(),
  }
}

/// Команда остановки главного процесса
#[tauri::command]
pub fn exit() {
  std::process::exit(0x0);
}

/// Команда проверки, запущен ли клиент в режиме разработки
#[tauri::command]
pub fn is_dev_mode() -> bool {
  let dev_flag = "dev".to_string();
  std::env::args().collect::<Vec<String>>().contains(&dev_flag)
}

/// Команда задания статуса Discord RPC
#[tauri::command]
pub async fn set_discord_rpc(version: String, state: bool) {
  if state {
    DISCORD_RPC.write().await.enable(version);
  } else {
    DISCORD_RPC.write().await.disable();
  }
}
