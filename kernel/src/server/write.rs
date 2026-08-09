use std::sync::Arc;

use bytes::Bytes;
use hashbrown::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;

/// Вспомогательная функция записи данных во все сокеты
pub async fn write_bytes_to_sockets(
  sockets: &Arc<RwLock<HashMap<String, (Arc<RwLock<OwnedReadHalf>>, Arc<RwLock<OwnedWriteHalf>>)>>>,
  bytes: Bytes,
) {
  for (addr, (_, w)) in sockets.read().await.iter() {
    match w.write().await.write_all(&bytes).await {
      Ok(_) => {}
      Err(e) => {
        println!("[error :: {}] data write error: {}", addr, e);
      }
    }
  }
}
