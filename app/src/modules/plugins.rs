use std::env::home_dir;
use std::fs;

use crate::result::CommandResult;
use crate::{failed, success};

/// Команда инициализации директории плагинов
#[tauri::command]
pub fn initialize_plugins_dir() -> CommandResult<()> {
  let Some(home_dir) = home_dir() else {
    failed!("failed to get home directory");
  };

  let salarixi_dir = home_dir.join(".salarixi");
  if !salarixi_dir.exists() {
    if fs::create_dir_all(&salarixi_dir).is_err() {
      failed!("salarixi directory is missing");
    }
  }

  let plugins_dir = salarixi_dir.join("plugins");
  if !plugins_dir.exists() {
    if fs::create_dir_all(plugins_dir).is_err() {
      failed!("plugins directory is missing");
    }
  }

  success!(());
}
