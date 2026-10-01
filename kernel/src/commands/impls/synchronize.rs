use std::sync::atomic::Ordering;
use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use bytes::{BufMut, Bytes, BytesMut};
use salarixi_extensions::buffer::BufferExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use crate::bot::systems::index::INDEX_SYSTEM;
use crate::launch::options::{CaptchaSolveMode, CaptchaType};
use crate::launch::process::{current_options, process_status_payload, PROCESS_ACTIVITY};
use crate::server::{write_to_socket, write_to_sockets, Sockets};

async fn create_sync_buf() -> Option<Bytes> {
  let process = process_status_payload().await;

  let Some(opts) = current_options().await else {
    return None;
  };

  let mut buf = BytesMut::new();
  buf.put_u8(0x0C);
  buf.put_u8(if PROCESS_ACTIVITY.load(Ordering::SeqCst) {
    0x01
  } else {
    0x02
  });
  process.status_id.write(&mut buf);
  process.connected_bots.write(&mut buf);
  process.total_bots.write(&mut buf);
  buf.put_u8(if opts.basic.use_anti_captcha { 0x01 } else { 0x00 });
  buf.put_u8(match opts.captcha_bypass.captcha_type {
    CaptchaType::Web => 0x00,
    CaptchaType::Map => 0x01,
  });
  buf.put_u8(match opts.captcha_bypass.solve_mode {
    CaptchaSolveMode::Manual => 0x00,
    CaptchaSolveMode::Auto => 0x01,
  });
  opts.basic.enable_monitoring.write(&mut buf);
  opts.basic.chat_monitoring.write(&mut buf);
  opts.basic.inventory_monitoring.write(&mut buf);

  if PROCESS_ACTIVITY.load(Ordering::SeqCst) {
    let map = INDEX_SYSTEM.map.read().await;
    buf.put_u8(map.len() as u8);

    for username in map.iter() {
      username.write(&mut buf);
    }
  }

  Some(buf.freeze())
}

pub async fn synchronize(addr: &str, writer: &Arc<RwLock<OwnedWriteHalf>>, cipher: &Arc<Aes256Gcm>) {
  let Some(buf) = create_sync_buf().await else {
    return;
  };

  write_to_socket(addr, writer, cipher, &buf).await;
}

pub async fn synchronize_all(sockets: &Sockets) {
  let Some(buf) = create_sync_buf().await else {
    return;
  };

  write_to_sockets(sockets, &buf).await;
}
