use std::sync::{Arc, LazyLock};

use aes_gcm::Aes256Gcm;
use azalea::inventory::Menu;
use bytes::{BufMut, Bytes, BytesMut};
use hashbrown::HashMap;
use md5::digest::Update;
use md5::{Digest, Md5};
use salarixi_extensions::buffer::BufferExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use crate::bot::extensions::BotInventoryExt;
use crate::launch::process::current_options;
use crate::server::write_to_socket;
use crate::take_bot;

pub static INV_HASH_TABLE: LazyLock<RwLock<HashMap<u8, [u8; 16]>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

fn create_menu_hash(menu: &Menu) -> Option<[u8; 16]> {
  let mut buf = BytesMut::new();

  for item in menu.slots() {
    (item.kind() as u16).write(&mut buf);

    let count = item.count();

    if count > 64 {
      0x41.write(&mut buf);
    } else {
      (count as u8).write(&mut buf);
    }
  }

  let finalized = Md5::new().chain(&buf).finalize();
  let hash: [u8; 16] = match finalized.as_slice()[..16].try_into() {
    Ok(h) => h,
    Err(_) => return None,
  };

  Some(hash)
}

pub async fn update_inventory(
  addr: &str,
  writer: &Arc<RwLock<OwnedWriteHalf>>,
  cipher: &Arc<Aes256Gcm>,
  data: &mut Bytes,
) -> Option<()> {
  let Some(opts) = current_options().await else {
    return Some(());
  };

  if !opts.basic.enable_monitoring || !opts.basic.inventory_monitoring {
    return Some(());
  }

  let index = u8::read(data)?;

  take_bot!(&index, async |bot| {
    let Some(menu) = bot.get_inventory_menu() else {
      return;
    };

    if menu.len() != 46 {
      return;
    }

    if opts.basic.monitoring_optimization {
      let mut hash_table = INV_HASH_TABLE.write().await;

      if let Some(hash) = hash_table.get_mut(&index) {
        if let Some(c_hash) = create_menu_hash(&menu) {
          if c_hash == *hash {
            return;
          }

          *hash = c_hash;
        }
      } else {
        if let Some(c_hash) = create_menu_hash(&menu) {
          hash_table.insert(index, c_hash);
        }
      }
    }

    let mut buf = BytesMut::new();
    buf.put_u8(0x0F);
    index.write(&mut buf);
    (menu.slots().len() as u8).write(&mut buf);

    for (slot, item) in menu.slots().iter().enumerate() {
      (slot as u8).write(&mut buf);
      (if item.count() > 64 { 65 } else { item.count() as u8 }).write(&mut buf);

      if item.is_empty() {
        None::<String>.write(&mut buf);
      } else {
        Some(item.kind().to_string()[10..].to_string()).write(&mut buf);
      }
    }

    write_to_socket(addr, writer, cipher, &buf).await;
  });

  Some(())
}
