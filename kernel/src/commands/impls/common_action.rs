use bytes::Bytes;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::common_actions::{disconnect_bot, reset_bot};
use crate::take_bot;

pub async fn common_action(data: &mut Bytes) -> Option<()> {
  let action = u8::read(data)?;
  let index = u8::read(data)?;

  match action {
    0x00 => {
      let message = String::read(data)?;
      take_bot!(&index, async |bot| bot.chat(message));
    }
    0x01 => {
      reset_bot(&index, false).await;
    }
    0x02 => {
      disconnect_bot(&index, false).await;
    }
    _ => {}
  }

  Some(())
}
