use bytes::{BufMut, Bytes, BytesMut};
use hashbrown::HashMap;
use salarixi_extensions::buffer::BufferExt;

use crate::bot::systems::profile::Profile;

#[derive(Clone)]
pub struct LogPayload {
  pub text: String,
  pub class: String,
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
  pub receiver: String,
  pub message: String,
}

impl BotChatPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x04);
    self.receiver.write(&mut buf);
    self.message.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct AntiWebCaptchaPayload {
  pub captcha_url: String,
  pub username: String,
}

impl AntiWebCaptchaPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x05);
    self.captcha_url.write(&mut buf);
    self.username.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct AntiMapCaptchaPayload {
  pub base64_code: String,
  pub username: String,
}

impl AntiMapCaptchaPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x06);
    self.base64_code.write(&mut buf);
    self.username.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct RemoveCaptchaPayload {
  pub username: String,
}

impl RemoveCaptchaPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x07);
    self.username.write(&mut buf);
    buf.freeze()
  }
}

#[derive(Clone)]
pub struct MonitoringProfilesPayload {
  pub profiles: HashMap<String, Profile>,
}

impl MonitoringProfilesPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u8(0x08);
    buf.put_u8(self.profiles.len() as u8);

    for (username, profile) in &self.profiles {
      username.write(&mut buf);
      profile.write(&mut buf);
    }

    buf.freeze()
  }
}
