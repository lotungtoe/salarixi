use std::sync::LazyLock;

use tokio::sync::broadcast;

use crate::server::transfer::{
  AntiMapCaptchaPayload, AntiWebCaptchaPayload, BotChatPayload, LogPayload, MessagePayload, ProcessStatusPayload,
  SessionChatPayload, UpdateBotProfilePayload,
};

pub static TRANSFER: LazyLock<TransferManager> = LazyLock::new(|| TransferManager::new());

#[derive(Clone)]
pub enum TransferEvent {
  Log(LogPayload),
  Message(MessagePayload),
  SessionChat(SessionChatPayload),
  BotChat(BotChatPayload),
  AntiWebCaptcha(AntiWebCaptchaPayload),
  AntiMapCaptcha(AntiMapCaptchaPayload),
  UpdateBotProfile(UpdateBotProfilePayload),
  ProcessStatus(ProcessStatusPayload),
}

pub struct TransferManager {
  pub tx: broadcast::Sender<TransferEvent>,
}

impl TransferManager {
  pub fn new() -> Self {
    let (tx, _) = broadcast::channel(255);

    Self { tx }
  }

  pub fn emit(&self, event: TransferEvent) {
    if self.tx.receiver_count() > 0 {
      let _ = self.tx.send(event);
    }
  }
}

pub enum LogClass {
  Info,
  Warning,
  System,
  Extended,
  Error,
  NcError,
}

impl LogClass {
  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "info" => Some(Self::Info),
      "warning" => Some(Self::Warning),
      "system" => Some(Self::System),
      "extended" => Some(Self::Extended),
      "error" => Some(Self::Error),
      "nc-error" => Some(Self::NcError),
      _ => None,
    }
  }

  fn to_index(&self) -> u8 {
    match self {
      Self::Info => 0x00,
      Self::Warning => 0x01,
      Self::System => 0x02,
      Self::Extended => 0x03,
      Self::Error => 0x04,
      Self::NcError => 0x05,
    }
  }
}

/// Вспомогательная функция отправки лога
pub fn emit_log(text: impl Into<String>, class: LogClass) {
  TRANSFER.emit(TransferEvent::Log(LogPayload {
    text: text.into(),
    class: class.to_index(),
  }));
}

/// Вспомогательная функция отправки сообщения
pub fn emit_msg(name: impl Into<String>, content: impl Into<String>) {
  TRANSFER.emit(TransferEvent::Message(MessagePayload {
    name: name.into(),
    content: content.into(),
  }));
}
