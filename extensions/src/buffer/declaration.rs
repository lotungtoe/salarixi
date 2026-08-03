use bytes::BytesMut;

pub trait BufferExt
where
  Self: Sized,
{
  fn read(buf: &mut BytesMut) -> Option<Self>;
}
