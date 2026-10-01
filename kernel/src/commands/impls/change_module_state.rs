use bytes::Bytes;

use crate::bot::MODULES;
use crate::launch::process::process_is_active;

pub async fn change_module_state(data: Bytes) {
  if process_is_active() {
    MODULES.control(data).await;
  }
}
