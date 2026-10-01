use bytes::{BufMut, BytesMut};

use crate::launch::runner::stop_bots_and_destroy_data;
use crate::server::{write_to_sockets, Sockets};

use super::synchronize_all;

pub async fn stop_bots(sockets: &Sockets) {
  let status = stop_bots_and_destroy_data(true).await;

  let mut buf = BytesMut::new();
  buf.put_u8(0x0A);
  buf.put_u8(status);

  write_to_sockets(sockets, &buf).await;
  synchronize_all(sockets).await;
}
