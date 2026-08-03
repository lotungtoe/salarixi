use bytes::Buf;

use crate::buffer::BufferExt;

impl BufferExt for i32 {
  fn read(buf: &mut bytes::BytesMut) -> Option<Self> {
    buf.try_get_i32().ok()
  }
}
