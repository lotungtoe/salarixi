use bytes::Bytes;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::common::convert_inventory_slot_to_hotbar_slot;
use crate::bot::extensions::{BotInventoryExt, ClickMode};
use crate::take_bot;

pub async fn inventory_action(data: &mut Bytes) -> Option<()> {
  let index = u8::read(data)?;
  let slot = u8::read(data)?;
  let action = u8::read(data)?;

  let mut result = Some(());

  take_bot!(&index, async |bot| {
    match action {
      0x00 => {
        let Some(hotbar_slot) = convert_inventory_slot_to_hotbar_slot(slot as usize) else {
          return;
        };

        bot.set_selected_hotbar_slot(hotbar_slot);
      }
      0x01 => {
        bot.inventory_click(&index, slot as usize, ClickMode::Drop, true).await;
      }
      0x02 => {
        bot
          .inventory_click(&index, slot as usize, ClickMode::DropAll, true)
          .await;
      }
      0x03 => {
        bot.inventory_click(&index, slot as usize, ClickMode::Shift, true).await;
      }
      _ => result = None,
    }
  });

  result
}
