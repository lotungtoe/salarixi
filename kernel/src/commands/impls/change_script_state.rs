use bytes::Bytes;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::script::SCRIPT_EXECUTOR;
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::launch::process::process_is_active;

pub async fn execute_script(data: &mut Bytes) -> Option<()> {
  let script = String::read(data)?;
  let separately = bool::read(data)?;

  if !process_is_active() {
    return Some(());
  }

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

  Some(())
}

pub async fn stop_script() {
  SCRIPT_EXECUTOR.stop();
}
