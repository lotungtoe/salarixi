use bytes::{BufMut, Bytes, BytesMut};

use crate::server::{write_to_sockets, Sockets};

pub async fn remove_captcha(data: &Bytes, sockets: &Sockets) {
  let mut buf = BytesMut::new();
  buf.put_u8(0x07);
  buf.put_slice(data);

  write_to_sockets(sockets, &buf).await;
}
