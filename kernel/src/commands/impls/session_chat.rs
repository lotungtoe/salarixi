use bytes::{BufMut, Bytes, BytesMut};
use salarixi_extensions::buffer::BufferExt;

use crate::server::{write_to_sockets, Sockets};

pub async fn session_chat(addr: String, data: &Bytes, sockets: &Sockets) {
  let mut buf = BytesMut::new();
  buf.put_u8(0x02);
  addr.write(&mut buf);
  buf.put_slice(data);

  write_to_sockets(sockets, &buf).await;
}
