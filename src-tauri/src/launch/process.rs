use std::sync::atomic::{AtomicBool, Ordering};

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use super::options::LaunchOptions;

pub static PROCESS_ACTIVITY: AtomicBool = AtomicBool::new(false);
pub static CURRENT_OPTIONS: Lazy<RwLock<Option<LaunchOptions>>> = Lazy::new(|| RwLock::new(None));

/// Функция установки опций
pub async fn set_options(options: LaunchOptions) {
  *CURRENT_OPTIONS.write().await = Some(options);
}

/// Функция получения текущих опций
pub async fn current_options() -> Option<LaunchOptions> {
  CURRENT_OPTIONS.read().await.clone()
}

/// Функция установки значения активности процесса
pub fn set_process_activity(value: bool) {
  PROCESS_ACTIVITY.store(value, Ordering::SeqCst);
}

/// Функция проверки активности основного процесса
pub fn process_is_active() -> bool {
  PROCESS_ACTIVITY.load(Ordering::SeqCst)
}
