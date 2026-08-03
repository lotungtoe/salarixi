use bytes::Buf;

use crate::buffer::BufferExt;

impl BufferExt for f32 {
  fn read(buf: &mut bytes::BytesMut) -> Option<Self> {
    buf.try_get_f32().ok()
  }
}

impl BufferExt for f64 {
  fn read(buf: &mut bytes::BytesMut) -> Option<Self> {
    buf.try_get_f64().ok()
  }
}
