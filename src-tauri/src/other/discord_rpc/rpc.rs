use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::emit::emit_log;

pub static DISCORD_RPC: Lazy<RwLock<DiscordRpc>> = Lazy::new(|| RwLock::new(DiscordRpc::new()));

pub struct DiscordRpc {
  client: Option<discord_presence::Client>,
}

impl DiscordRpc {
  pub fn new() -> Self {
    Self { client: None }
  }

  pub fn enable(&mut self, version: String) {
    self.client = Some(discord_presence::Client::new(1477312950271213729));

    if let Some(client) = self.client.as_mut() {
      client.start();
      let _ = client.block_until_event(discord_presence::Event::Ready);

      match client.set_activity(|act| {
        act
          .details(format!("Версия: {}", version))
          .state("Сайт: https://salarixi.freedev.app/")
      }) {
        Ok(_) => {}
        Err(err) => {
          emit_log(format!("Ошибка включения Discord RPC: {}", err), "error");
        }
      }
    }
  }

  pub fn disable(&mut self) {
    if let Some(client) = self.client.take() {
      match client.shutdown() {
        Ok(_) => {}
        Err(err) => {
          emit_log(format!("Ошибка выключения Discord RPC: {}", err), "error");
        }
      }
    }
  }
}
