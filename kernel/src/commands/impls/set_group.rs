use bytes::{BufMut, Bytes, BytesMut};
use salarixi_extensions::buffer::BufferExt;

use crate::server::{write_to_sockets, Sockets};
use crate::take_profile;

pub async fn set_group(data: &mut Bytes, sockets: &Sockets) -> Option<()> {
  let index = u8::read(data)?;
  let group = String::read(data)?;

  take_profile!(&index, async |profile| {
    profile.group = group.clone();
  });

  let mut buf = BytesMut::new();
  buf.put_u8(0x0E);
  index.write(&mut buf);
  group.write(&mut buf);

  write_to_sockets(sockets, &buf).await;

  Some(())
}
