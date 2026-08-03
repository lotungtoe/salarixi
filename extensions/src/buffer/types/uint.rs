use bytes::Buf;

use crate::buffer::BufferExt;

impl BufferExt for u8 {
  fn read(buf: &mut bytes::BytesMut) -> Option<Self> {
    buf.try_get_u8().ok()
  }
}

impl BufferExt for u16 {
  fn read(buf: &mut bytes::BytesMut) -> Option<Self> {
    buf.try_get_u16().ok()
  }
}

impl BufferExt for u64 {
  fn read(buf: &mut bytes::BytesMut) -> Option<Self> {
    buf.try_get_u64().ok()
  }
}
