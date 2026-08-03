use bytes::Buf;

use crate::buffer::BufferExt;

impl<T> BufferExt for Option<T>
where
  T: BufferExt,
{
  fn read(buf: &mut bytes::BytesMut) -> Option<Self> {
    let is_some = buf.try_get_u8().ok()? == 1;

    if is_some {
      return Some(T::read(buf));
    }

    // Хахаха, почему-то это выглядит смешно...
    Some(None)
  }
}
