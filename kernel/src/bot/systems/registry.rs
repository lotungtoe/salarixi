use std::sync::{Arc, LazyLock};

use azalea::prelude::*;
use azalea::swarm::Swarm;
use dashmap::DashMap;
use tokio::sync::RwLock;

use crate::server::transfer::{emit_log, LogClass};

pub static REGISTRY_SYSTEM: LazyLock<RegistrySystem> = LazyLock::new(|| RegistrySystem::new());

pub struct RegistrySystem {
  pub swarm: Arc<RwLock<Option<Swarm>>>,
  pub bots: DashMap<u8, Arc<Client>>,
}

impl RegistrySystem {
  pub fn new() -> Self {
    Self {
      swarm: Arc::new(RwLock::new(None)),
      bots: DashMap::new(),
    }
  }

  /// Метод установки роя
  pub async fn set_swarm(&self, swarm: Swarm) {
    let mut guard = self.swarm.write().await;
    *guard = Some(swarm);
  }

  /// Метод регистрации бота
  pub fn register_bot(&self, index: u8, bot: Client) {
    self.bots.insert(index, Arc::new(bot));
  }

  /// Метод получения бота по юзернейму
  pub fn get_bot(&self, index: &u8) -> Option<Arc<Client>> {
    self.bots.get(index).map(|cell| cell.clone())
  }

  /// Асинхронный метод получения бота по юзернейму
  pub async fn async_get_bot<F, T>(&self, index: &u8, f: F) -> Option<T>
  where
    F: AsyncFnOnce(&Client) -> T,
  {
    if let Some(reference) = self.bots.get(index) {
      return Some(f(reference.as_ref()).await);
    }

    None
  }

  /// Метод полной очистки реестра
  pub async fn clear(&self) {
    self.bots.clear();
    self.bots.shrink_to_fit();

    if let Some(swarm) = self.swarm.write().await.take() {
      swarm.exit();
    }

    emit_log("Реестр ботов очищен", LogClass::System);
  }
}

/// Вспомогательный макрос захвата указанного бота
#[macro_export]
macro_rules! take_bot {
  ($index:expr, $func:expr) => {
    crate::bot::systems::registry::REGISTRY_SYSTEM
      .async_get_bot($index, $func)
      .await;
  };
}
