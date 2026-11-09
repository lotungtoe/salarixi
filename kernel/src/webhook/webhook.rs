use serde::{Deserialize, Serialize};

use crate::server::transfer::*;

#[derive(Serialize, Deserialize)]
struct Webhook {
  content: String,
}

pub fn send_webhook(webhook_url: Option<String>, content: String) {
  tokio::spawn(async move {
    if let Some(url) = webhook_url {
      let msg = Webhook { content: content };

      let client = reqwest::Client::new();

      let res = client.post(url).json(&msg).send().await;

      match res {
        Ok(r) => {
          if !r.status().is_success() {
            emit_log(
              format!("Không thể gửi webhook: mã trạng thái {}", r.status()),
              LogClass::Error,
            );
          }
        }
        Err(err) => {
          emit_log(format!("Không thể gửi webhook: {}", err), LogClass::Error);
        }
      }
    }
  });
}
