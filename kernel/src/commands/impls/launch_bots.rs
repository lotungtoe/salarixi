use bytes::{BufMut, Bytes, BytesMut};

use crate::launch::options::LaunchOptions;
use crate::launch::runner::launch_bots_on_server;
use crate::server::{write_to_sockets, Sockets};

use super::synchronize_all;

pub async fn launch_bots(data: &mut Bytes, sockets: &Sockets) {
  let options = match LaunchOptions::read(data) {
    Some(o) => o,
    None => {
      let mut buf = BytesMut::new();
      buf.put_u8(0x09);
      buf.put_u8(0x00);

      write_to_sockets(sockets, &buf).await;
      synchronize_all(sockets).await;

      return;
    }
  };

  let status = launch_bots_on_server(options).await;

  let mut buf = BytesMut::new();
  buf.put_u8(0x09);
  buf.put_u8(status);

  write_to_sockets(sockets, &buf).await;
  synchronize_all(sockets).await;
}
