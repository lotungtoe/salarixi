use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use super::options::LaunchOptions;

pub static ACTIVE: AtomicBool = AtomicBool::new(false);
pub static CURRENT_OPTIONS: Lazy<Arc<RwLock<Option<LaunchOptions>>>> = Lazy::new(|| Arc::new(RwLock::new(None)));

/// Функция установки опций
pub fn set_options(options: LaunchOptions) {
  let mut guard = CURRENT_OPTIONS.write().unwrap();
  *guard = Some(options);
}

/// Функция получения текущих опций
pub fn current_options() -> Option<LaunchOptions> {
  CURRENT_OPTIONS.read().unwrap().clone()
}

/// Функция проверки активности основного процесса
pub fn process_is_active() -> bool {
  ACTIVE.load(Ordering::SeqCst)
}
