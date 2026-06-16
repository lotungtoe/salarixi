use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

pub static EMIT_MANAGER: Lazy<Arc<EmitManager>> = Lazy::new(|| Arc::new(EmitManager::new()));

#[derive(Serialize, Clone)]
pub struct LogPayload {
  pub text: String,
  pub class: String,
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
  Optional(OptionalEmitEvent),
}

#[derive(Clone)]
pub enum OptionalEmitEvent {
  Chat(ChatEventPayload),
  AntiWebCaptcha(AntiWebCaptchaEventPayload),
  AntiMapCaptcha(AntiMapCaptchaEventPayload),
}

impl EmitManager {
  pub fn new() -> Self {
    let (tx, _) = broadcast::channel(1000);

    Self { events: tx }
  }

  pub fn send_event(&self, event: EmitEvent) {
    let _ = self.events.send(event);
  }
}

/// Вспомогательная функция отправки лога
pub fn emit_log(text: impl Into<String>, class: &str) {
  EMIT_MANAGER.send_event(EmitEvent::Log {
    text: text.into(),
    class: class.to_string(),
  });
}

/// Вспомогательная функция отправки сообщения
pub fn emit_msg(name: &str, content: impl Into<String>) {
  EMIT_MANAGER.send_event(EmitEvent::Message {
    name: name.to_string(),
    content: content.into(),
  });
}

/// Вспомогательная функция отправки дополнительных событий
pub fn send_optional_event(event: OptionalEmitEvent) {
  match event {
    OptionalEmitEvent::Chat(payload) => {
      EMIT_MANAGER.send_event(EmitEvent::Optional(OptionalEmitEvent::Chat(payload)));
    }
    OptionalEmitEvent::AntiWebCaptcha(payload) => {
      EMIT_MANAGER.send_event(EmitEvent::Optional(OptionalEmitEvent::AntiWebCaptcha(payload)));
    }
    OptionalEmitEvent::AntiMapCaptcha(payload) => {
      EMIT_MANAGER.send_event(EmitEvent::Optional(OptionalEmitEvent::AntiMapCaptcha(payload)));
    }
  }
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
      EmitEvent::Optional(optional_event) => match optional_event {
        OptionalEmitEvent::Chat(payload) => {
          let _ = handle.emit("chat-message", payload);
        }
        OptionalEmitEvent::AntiWebCaptcha(payload) => {
          let _ = handle.emit("anti-web-captcha", payload);
        }
        OptionalEmitEvent::AntiMapCaptcha(payload) => {
          let _ = handle.emit("anti-map-captcha", payload);
        }
      },
    }
  }
}

/*
/// Вспомогательная функция отправки чего-то
pub fn emit_something<'e, P>(app: &AppHandle, event: impl Into<&'e str>, payload: P)
where
  P: Serialize + Clone,
{
  let _ = app.emit(event.into(), payload);
}
*/
