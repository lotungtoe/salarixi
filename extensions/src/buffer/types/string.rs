use bytes::Buf;

use crate::buffer::BufferExt;

impl BufferExt for String {
  fn read(buf: &mut bytes::BytesMut) -> Option<Self> {
    // Долго над этим не думал, пусть будет u16, маловероятно
    // что кому-то не хватит на одну строку 65535 символа
    let len = buf.try_get_u16().ok()?;
    let mut arr = Vec::with_capacity(len.into());

    for _ in 0..len {
      let byte = buf.try_get_u8().ok()?;
      arr.push(byte);
    }

    Some(String::from_utf8_lossy(&arr).into())
  }
}
