use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use hashbrown::HashMap;

use crate::other::tools::{randstr, CharClass};
use crate::result::CommandResult;
use crate::{failed, success};

/// Команда инициализации директории пользовательских конфигов
#[tauri::command]
pub fn initialize_configs_dir() -> CommandResult<()> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let configs_dir = salarixi_dir.join("configs");
  if !configs_dir.exists() {
    if fs::create_dir_all(configs_dir).is_err() {
      failed!("configs directory is missing");
    }
  }

  success!(());
}

/// Команда загрузки всех пользовательских конфигов
#[tauri::command]
pub fn load_configs() -> CommandResult<HashMap<String, Vec<u8>>> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let configs_dir = salarixi_dir.join("configs");
  if !configs_dir.exists() {
    if fs::create_dir_all(&configs_dir).is_err() {
      failed!("configs directory is missing");
    }
  }

  let mut configs = HashMap::new();

  let entries = match configs_dir.read_dir() {
    Ok(e) => e,
    Err(e) => failed!("config directory read error: {}", e),
  };

  for entry_result in entries {
    let Ok(entry) = entry_result else {
      continue;
    };

    let Ok(ty) = entry.file_type() else {
      continue;
    };

    if !ty.is_file() {
      continue;
    }

    let path = entry.path();

    let Some(ext) = path.extension() else {
      continue;
    };

    if ext != "json" {
      continue;
    }

    let os_filename = entry.file_name();
    let filename = String::from_utf8_lossy(os_filename.as_bytes());
    let content = match fs::read(path) {
      Ok(c) => c,
      Err(_) => continue,
    };

    configs.insert(filename.to_string(), content);
  }

  success!(configs);
}

/// Команда создания публичного конфига
#[tauri::command]
pub fn create_public_config(directory: String) -> CommandResult<String> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let config_path = salarixi_dir.join("current_config.json");
  let bytes = match fs::read(config_path) {
    Ok(b) => b,
    Err(e) => failed!("current config read error: {}", e),
  };

  let filename = format!("config_{}.json", randstr(CharClass::Numeric, 4));

  let config_dir = PathBuf::from(directory);
  let public_config_path = config_dir.join(&filename);
  match fs::write(public_config_path, &bytes) {
    Ok(_) => {}
    Err(e) => failed!("error creating public config: {}", e),
  };

  success!(filename);
}
