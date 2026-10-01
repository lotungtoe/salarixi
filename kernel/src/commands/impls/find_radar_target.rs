use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use bytes::{BufMut, Bytes, BytesMut};
use salarixi_extensions::buffer::BufferExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use crate::bot::utils::radar::RADAR;
use crate::server::write_to_socket;

pub async fn find_radar_target(
  data: &mut Bytes,
  addr: String,
  writer: &Arc<RwLock<OwnedWriteHalf>>,
  cipher: &Arc<Aes256Gcm>,
) -> Option<()> {
  let target = String::read(data)?;

  if let Some(info) = RADAR.find_target(target.clone()).await {
    let mut buf = BytesMut::new();
    buf.put_u8(0x0B);
    target.write(&mut buf);
    info.uuid.write(&mut buf);
    info.tx.write(&mut buf);
    info.ty.write(&mut buf);
    info.tz.write(&mut buf);
    info.ox.write(&mut buf);
    info.oz.write(&mut buf);

    write_to_socket(&addr, writer, cipher, &buf).await;
  }

  Some(())
}
