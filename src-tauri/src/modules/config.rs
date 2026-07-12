use std::fs;
use std::path::PathBuf;

use crate::result::CommandResult;
use crate::{failed, success};

/// Команда создания публичного конфига
#[tauri::command]
pub fn create_public_config(directory: String) -> CommandResult<()> {
  let Some(home_dir) = dirs::home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let config_path = salarixi_dir.join("config.json");
  let bytes = match fs::read(config_path) {
    Ok(b) => b,
    Err(e) => failed!("config read error: {}", e),
  };

  let config_dir = PathBuf::from(directory);
  let public_config_path = config_dir.join("salarixi.config.json");
  match fs::write(public_config_path, &bytes) {
    Ok(_) => {}
    Err(e) => failed!("error creating public config: {}", e),
  };

  success!(());
}
