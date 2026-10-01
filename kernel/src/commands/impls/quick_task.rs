use bytes::Bytes;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::quick::run_quick_task;

pub async fn quick_task(data: &mut Bytes) -> Option<()> {
  let id = u8::read(data)?;

  run_quick_task(id).await;

  Some(())
}
