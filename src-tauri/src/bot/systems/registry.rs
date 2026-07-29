use std::sync::Arc;

use azalea::prelude::*;
use azalea::swarm::Swarm;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::sync::{broadcast, RwLock};

use crate::bot::quick::QUICK_TASKS;
use crate::bot::{MODULES, PLUGINS};
use crate::emit::emit_log;
use crate::launch::process::current_options;

pub static REGISTRY_SYSTEM: Lazy<RegistrySystem> = Lazy::new(|| RegistrySystem::new());

/// Система реестра ботов и их событий
pub struct RegistrySystem {
  pub swarm: Arc<RwLock<Option<Swarm>>>,
  pub bots: DashMap<u8, Arc<Client>>,
  pub events: broadcast::Sender<RegistryEvent>,
}

#[derive(Clone)]
pub enum RegistryEvent {
  ActivatePlugins {
    index: u8,
  },
  ControlModules {
    name: String,
    options: serde_json::Value,
    group: String,
  },
  QuickTask {
    name: String,
  },
  DestroySwarm,
  StopProcessing,
}

impl RegistrySystem {
  /// Метод создания нового реестра
  pub fn new() -> Self {
    let (tx, _) = broadcast::channel(100);

    Self {
      swarm: Arc::new(RwLock::new(None)),
      bots: DashMap::new(),
      events: tx,
    }
  }

  /// Метод установки роя
  pub async fn set_swarm(&self, swarm: Swarm) {
    let mut guard = self.swarm.write().await;
    *guard = Some(swarm);
  }

  /// Метод уничтожения роя
  pub async fn destroy_swarm(&self) {
    if let Some(swarm) = self.swarm.write().await.take() {
      swarm.ecs_lock.lock().write_message(AppExit::Success);
    }
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

  /// Метод отправки события в реестр
  pub fn send_event(&self, event: RegistryEvent) {
    let _ = self.events.send(event);
  }

  /// Метод полной очистки реестра
  pub fn clear(&self) {
    self.bots.clear();
    self.send_event(RegistryEvent::DestroySwarm);
    self.send_event(RegistryEvent::StopProcessing);

    emit_log("Реестр ботов очищен", "extended");
  }
}

/// Функция процессинга всех событий из `RegistryEvent`
pub async fn registry_event_loop() {
  let mut rx = REGISTRY_SYSTEM.events.subscribe();

  while let Ok(event) = rx.recv().await {
    match event {
      RegistryEvent::ActivatePlugins { index } => {
        if let Some(opts) = current_options().await {
          PLUGINS.activate_for(index, opts.plugins).await;
        }
      }
      RegistryEvent::ControlModules { name, options, group } => {
        MODULES.control(name, options, group).await;
      }
      RegistryEvent::QuickTask { name } => {
        QUICK_TASKS.execute(name).await;
      }
      RegistryEvent::DestroySwarm => {
        REGISTRY_SYSTEM.destroy_swarm().await;
      }
      RegistryEvent::StopProcessing => {
        return;
      }
    }
  }
}

/// Вспомогательный макрос захвата указанного бота
#[macro_export]
macro_rules! take_bot {
  ($username:expr, $func:expr) => {
    crate::bot::systems::registry::REGISTRY_SYSTEM
      .async_get_bot($username, $func)
      .await;
  };
}
