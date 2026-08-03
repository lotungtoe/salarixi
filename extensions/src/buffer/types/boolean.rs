use bytes::Buf;

use crate::buffer::BufferExt;

impl BufferExt for bool {
  fn read(buf: &mut bytes::BytesMut) -> Option<Self> {
    Some(buf.try_get_u8().ok()? == 1)
  }
}
