use bytes::{BufMut, Bytes, BytesMut};
use salarixi_extensions::buffer::BufferExt;

use crate::bot::systems::profile::Profile;

#[derive(Clone)]
pub struct LogPayload {
  pub text: String,
  pub class: u8,
}

impl LogPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x00);
    self.text.write(&mut buf);
    self.class.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct MessagePayload {
  pub name: String,
  pub content: String,
}

impl MessagePayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x01);
    self.name.write(&mut buf);
    self.content.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct SessionChatPayload {
  pub sender: String,
  pub message: String,
}

impl SessionChatPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x02);
    self.sender.write(&mut buf);
    self.message.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct ProcessStatusPayload {
  pub status_id: u8,
  pub connected_bots: u8,
  pub total_bots: u8,
}

impl ProcessStatusPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::with_capacity(8);
    buf.put_u8(0x03);
    self.status_id.write(&mut buf);
    self.connected_bots.write(&mut buf);
    self.total_bots.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct BotChatPayload {
  pub index: u8,
  pub message: String,
}

impl BotChatPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x04);
    self.index.write(&mut buf);
    self.message.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct AntiWebCaptchaPayload {
  pub index: u8,
  pub captcha_url: String,
}

impl AntiWebCaptchaPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x05);
    self.index.write(&mut buf);
    self.captcha_url.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct AntiMapCaptchaPayload {
  pub index: u8,
  pub b64: String,
}

impl AntiMapCaptchaPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x06);
    self.index.write(&mut buf);

    if self.b64.len() < u16::MAX as usize {
      buf.put_u8(0x01);
      self.b64.write(&mut buf);
      return buf.freeze();
    }

    let chunk_size = u16::MAX as usize;
    let parts = (self.b64.len() + chunk_size - 1) / chunk_size;
    buf.put_u8(parts as u8);

    for i in 0..parts {
      let start = chunk_size * i;
      let end = std::cmp::min(chunk_size * (i + 1), self.b64.len());
      self.b64[start..end].to_string().write(&mut buf);
    }

    buf.freeze()
  }
}

#[derive(Clone)]
pub struct RemoveCaptchaPayload {
  pub index: u8,
}

impl RemoveCaptchaPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x07);
    self.index.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct UpdateBotProfilePayload {
  pub index: u8,
  pub profile: Profile,
}

impl UpdateBotProfilePayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x08);
    self.index.write(&mut buf);
    self.profile.write(&mut buf);
    buf.freeze()
  }
}
