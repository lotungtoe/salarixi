use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

use crate::bot::utils::scanner::ServerMeta;

pub static EMIT_MANAGER: Lazy<EmitManager> = Lazy::new(|| EmitManager::new());

#[derive(Serialize, Clone)]
pub struct LogPayload {
  pub text: String,
  pub class: String,
}

#[derive(Serialize, Clone)]
pub struct ConnectedBotsPayload {
  pub count: u8,
}

#[derive(Serialize, Clone)]
pub struct MessagePayload {
  pub name: String,
  pub content: String,
}

#[derive(Serialize, Clone)]
pub struct ChatEventPayload {
  pub receiver: String,
  pub message: String,
}

#[derive(Serialize, Clone)]
pub struct AntiWebCaptchaEventPayload {
  pub captcha_url: String,
  pub username: String,
}

#[derive(Serialize, Clone)]
pub struct AntiMapCaptchaEventPayload {
  pub base64_code: String,
  pub username: String,
}

pub struct EmitManager {
  events: broadcast::Sender<EmitEvent>,
}

#[derive(Clone)]
pub enum EmitEvent {
  Log { text: String, class: String },
  Message { name: String, content: String },
  ConnectedBots { count: u8 },
  BotChat(ChatEventPayload),
  AntiWebCaptcha(AntiWebCaptchaEventPayload),
  AntiMapCaptcha(AntiMapCaptchaEventPayload),
  PushScannedServer(ServerMeta),
  SetScannerStatus(u8),
  ScannerHostsScanned(usize, usize),
  ScannerServersFound(usize),
}

impl EmitManager {
  pub fn new() -> Self {
    let (tx, _) = broadcast::channel(255);

    Self { events: tx }
  }

  pub fn emit(&self, event: EmitEvent) {
    let _ = self.events.send(event);
  }
}

/// Вспомогательная функция отправки лога
pub fn emit_log(text: impl Into<String>, class: impl Into<String>) {
  EMIT_MANAGER.emit(EmitEvent::Log {
    text: text.into(),
    class: class.into(),
  });
}

/// Вспомогательная функция отправки сообщения
pub fn emit_msg(name: impl Into<String>, content: impl Into<String>) {
  EMIT_MANAGER.emit(EmitEvent::Message {
    name: name.into(),
    content: content.into(),
  });
}

/// Вспомогательная функция отправки количества подключённых ботов
pub fn emit_cbc(count: u8) {
  EMIT_MANAGER.emit(EmitEvent::ConnectedBots { count });
}

pub async fn emit_event_loop(handle: AppHandle) {
  let mut rx = EMIT_MANAGER.events.subscribe();

  while let Ok(event) = rx.recv().await {
    match event {
      EmitEvent::Log { text, class } => {
        let _ = handle.emit("log", LogPayload { text, class });
      }
      EmitEvent::Message { name, content } => {
        let _ = handle.emit("message", MessagePayload { name, content });
      }
      EmitEvent::ConnectedBots { count } => {
        let _ = handle.emit("connected-bots-count", ConnectedBotsPayload { count });
      }
      EmitEvent::BotChat(payload) => {
        let _ = handle.emit("chat-message", payload);
      }
      EmitEvent::AntiWebCaptcha(payload) => {
        let _ = handle.emit("anti-web-captcha", payload);
      }
      EmitEvent::AntiMapCaptcha(payload) => {
        let _ = handle.emit("anti-map-captcha", payload);
      }
      EmitEvent::PushScannedServer(meta) => {
        let _ = handle.emit("scanner:push-server", meta);
      }
      EmitEvent::SetScannerStatus(status) => {
        let _ = handle.emit("scanner:status", status);
      }
      EmitEvent::ScannerHostsScanned(current, total) => {
        let _ = handle.emit("scanner:hosts-scanned", [current, total]);
      }
      EmitEvent::ScannerServersFound(count) => {
        let _ = handle.emit("scanner:servers-found", count);
      }
    }
  }
}
