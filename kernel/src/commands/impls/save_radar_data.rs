use bytes::Bytes;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::utils::radar::RADAR;

pub async fn save_radar_data(data: &mut Bytes) -> Option<()> {
  let target = String::read(data)?;
  let path = String::read(data)?;
  let filename = String::read(data)?;
  let x = f64::read(data)?;
  let y = f64::read(data)?;
  let z = f64::read(data)?;

  RADAR.save_data(target, path, filename, x, y, z);

  Some(())
}
