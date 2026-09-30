use std::env::home_dir;
use std::io::Read;
use std::time::Duration;

use base64::Engine;
use salarixi_kernel::sleep;
use systemstat::Platform;
use tokio::fs::File;
use zip::ZipArchive;

use crate::result::CommandResult;
use crate::session::SESSION;
use crate::{failed, success};

#[tauri::command]
pub async fn setup_default_session() -> CommandResult<(String, String)> {
  SESSION.run_local_server().await
}

#[tauri::command]
pub async fn change_session(address: String, password: Option<String>) -> CommandResult<()> {
  SESSION.connect_to_remote_server(address, password, true).await
}

#[tauri::command]
pub async fn send_command(data: Vec<u8>) -> CommandResult<()> {
  SESSION.send_command(data).await
}

#[tauri::command]
pub async fn get_ram_usage() -> f64 {
  if let Some(usage) = memory_stats::memory_stats() {
    return (usage.physical_mem as f64 / 1024.0) / 1024.0;
  }

  0.0
}

#[tauri::command]
pub async fn get_cpu_usage() -> f32 {
  let system = systemstat::System::new();

  if let Ok(cpu) = system.cpu_load_aggregate() {
    sleep!(1000);

    if let Ok(load) = cpu.done() {
      return load.user * 100.0;
    }
  }

  0.0
}

#[tauri::command]
pub async fn open_url(url: String) {
  let _ = open::that(url);
}

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

#[tauri::command]
pub async fn download_text(url: String) -> Option<String> {
  let client = reqwest::Client::new();

  let resp = client.get(url).timeout(Duration::from_secs(10)).send().await.ok()?;

  if resp.status() != reqwest::StatusCode::OK {
    return None;
  }

  resp.text().await.ok()
}

#[tauri::command]
pub async fn extract_item_icon(texture: String) -> Vec<u8> {
  let Some(home_dir) = home_dir() else {
    return Vec::new();
  };

  let version_jar = home_dir
    .join(".minecraft")
    .join("versions")
    .join("1.21.11")
    .join("1.21.11.jar");

  if !version_jar.exists() {
    return Vec::new();
  }

  let Ok(file) = File::open(&version_jar).await else {
    return Vec::new();
  };

  let std_file = file.into_std().await;

  let Ok(mut jar) = ZipArchive::new(std_file) else {
    return Vec::new();
  };

  let mut buf = Vec::new();

  if let Ok(mut tf) = jar.by_name(&format!("assets/minecraft/textures/item/{}.png", texture)) {
    if tf.read_to_end(&mut buf).is_err() {
      return Vec::new();
    }
  }

  let parts = vec!["", "side", "top", "bottom"];

  for part in parts {
    if !buf.is_empty() {
      break;
    }

    let path = if part.is_empty() {
      format!("assets/minecraft/textures/block/{}.png", texture)
    } else {
      format!("assets/minecraft/textures/block/{}_{}.png", texture, part)
    };

    if let Ok(mut tf) = jar.by_name(&path) {
      if tf.read_to_end(&mut buf).is_err() {
        return Vec::new();
      }
    }
  }

  base64::engine::general_purpose::STANDARD.encode(&buf).into()
}

#[tauri::command]
pub fn read_file(path: String) -> Vec<u8> {
  match std::fs::read(&path) {
    Ok(b) => b,
    Err(_) => Vec::new(),
  }
}

#[tauri::command]
pub fn write_file(path: String, data: Vec<u8>) -> CommandResult<()> {
  match std::fs::write(&path, data) {
    Ok(_) => success!(()),
    Err(e) => failed!("{}", e),
  }
}

#[tauri::command]
pub fn exit() {
  std::process::exit(0x00);
}

#[tauri::command]
pub fn is_dev_mode() -> bool {
  let dev_flag = "dev".to_string();
  std::env::args().collect::<Vec<String>>().contains(&dev_flag)
}
