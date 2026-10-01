use bytes::Bytes;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::extensions::{BotInventoryExt, ClickMode};
use crate::take_bot;

pub async fn clear_inventory(data: &mut Bytes) -> Option<()> {
  let index = u8::read(data)?;

  take_bot!(&index, async |bot| {
    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, _) in menu.slots().iter().enumerate() {
        bot.inventory_click(&index, slot, ClickMode::DropAll, false).await;
      }
    }
  });

  Some(())
}
