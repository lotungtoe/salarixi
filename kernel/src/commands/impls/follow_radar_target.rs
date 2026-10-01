use bytes::Bytes;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::utils::radar::RADAR;

pub async fn follow_radar_target(data: &mut Bytes) -> Option<()> {
  let x = i32::read(data)?;
  let z = i32::read(data)?;

  RADAR.follow(x, z).await;

  Some(())
}
