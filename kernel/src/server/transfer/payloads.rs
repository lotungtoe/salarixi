use bytes::{BufMut, Bytes, BytesMut};
use salarixi_extensions::buffer::BufferExt;

#[derive(Clone)]
pub struct LogPayload {
  pub text: String,
  pub class: String,
}

impl LogPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    self.text.write(&mut buf);
    self.class.write(&mut buf);

    let mut final_buf = BytesMut::with_capacity(5 + buf.len());
    final_buf.put_u8(0x00);
    final_buf.put_u32(buf.len() as u32);
    final_buf.extend(buf);
    final_buf.freeze()
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
    self.name.write(&mut buf);
    self.content.write(&mut buf);

    let mut final_buf = BytesMut::with_capacity(5 + buf.len());
    final_buf.put_u8(0x01);
    final_buf.put_u32(buf.len() as u32);
    final_buf.extend(buf);
    final_buf.freeze()
  }
}

#[derive(Clone)]
pub struct ConnectedBotsPayload(pub u8);

impl ConnectedBotsPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    self.0.write(&mut buf);

    let mut final_buf = BytesMut::with_capacity(5 + buf.len());
    final_buf.put_u8(0x02);
    final_buf.put_u32(buf.len() as u32);
    final_buf.extend(buf);
    final_buf.freeze()
  }
}

#[derive(Clone)]
pub struct TotalBotsPayload(pub u8);

impl TotalBotsPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    self.0.write(&mut buf);

    let mut final_buf = BytesMut::with_capacity(5 + buf.len());
    final_buf.put_u8(0x03);
    final_buf.put_u32(buf.len() as u32);
    final_buf.extend(buf);
    final_buf.freeze()
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
    self.sender.write(&mut buf);
    self.message.write(&mut buf);

    let mut final_buf = BytesMut::with_capacity(5 + buf.len());
    final_buf.put_u8(0x04);
    final_buf.put_u32(buf.len() as u32);
    final_buf.extend(buf);
    final_buf.freeze()
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
    self.receiver.write(&mut buf);
    self.message.write(&mut buf);

    let mut final_buf = BytesMut::with_capacity(5 + buf.len());
    final_buf.put_u8(0x05);
    final_buf.put_u32(buf.len() as u32);
    final_buf.extend(buf);
    final_buf.freeze()
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
    self.captcha_url.write(&mut buf);
    self.username.write(&mut buf);

    let mut final_buf = BytesMut::with_capacity(5 + buf.len());
    final_buf.put_u8(0x06);
    final_buf.put_u32(buf.len() as u32);
    final_buf.extend(buf);
    final_buf.freeze()
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
    self.base64_code.write(&mut buf);
    self.username.write(&mut buf);

    let mut final_buf = BytesMut::with_capacity(5 + buf.len());
    final_buf.put_u8(0x07);
    final_buf.put_u32(buf.len() as u32);
    final_buf.extend(buf);
    final_buf.freeze()
  }
}

#[derive(Clone)]
pub struct RemoveCaptchaPayload {
  pub username: String,
}

impl RemoveCaptchaPayload {
  pub fn to_bytes(&self) -> Bytes {
    let mut buf = BytesMut::new();
    self.username.write(&mut buf);

    let mut final_buf = BytesMut::with_capacity(5 + buf.len());
    final_buf.put_u8(0x08);
    final_buf.put_u32(buf.len() as u32);
    final_buf.extend(buf);
    final_buf.freeze()
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
    buf.put_u8(0x0C);
    buf.put_u32(3);
    self.status_id.write(&mut buf);
    self.connected_bots.write(&mut buf);
    self.total_bots.write(&mut buf);
    buf.freeze()
  }
}
