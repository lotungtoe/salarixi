use azalea::prelude::*;
use azalea::swarm::Swarm;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::api::emit::emit_log;
use crate::service::core::launch::state::current_options;
use crate::service::core::module_manager::MODULE_MANAGER;
use crate::service::core::plugin_manager::PLUGINS;
use crate::service::quick::QUICK_TASK_MANAGER;

pub static BOT_REGISTRY: Lazy<Arc<BotRegistry>> = Lazy::new(|| Arc::new(BotRegistry::new()));

/// Реестр роя, всех ботов и событий.
/// Он выполняет роль основного хранилища всех ботов.
/// Так же данный реестр содержит в себе канал, по которому передаются все основные события для ботов (загрузить плагины, выполнить быструю задачу и так далее).
pub struct BotRegistry {
  pub swarm: Arc<RwLock<Option<Swarm>>>,
  pub bots: DashMap<String, Arc<Client>>,
  pub events: broadcast::Sender<RegistryEvent>,
}

#[derive(Clone)]
pub enum RegistryEvent {
  LoadPlugins {
    username: String,
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

impl BotRegistry {
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
  pub fn register_bot(&self, username: &str, bot: Client) {
    self.bots.insert(username.to_string(), Arc::new(bot));
  }

  /// Метод удаления бота из реестра
  pub async fn remove_bot(&self, username: &str) -> Option<Client> {
    let (_, cell) = self.bots.remove(username)?;
    Some((*cell).clone())
  }

  /// Метод получения бота по юзернейму
  pub fn get_bot(&self, username: &str) -> Option<Arc<Client>> {
    self.bots.get(username).map(|cell| cell.clone())
  }

  /// Асинхронный метод получения бота по юзернейму
  pub async fn async_get_bot<F, T>(&self, username: &str, f: F) -> Option<T>
  where
    F: AsyncFnOnce(&Client) -> T,
  {
    if let Some(reference) = self.bots.get(username) {
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
  let mut rx = BOT_REGISTRY.events.subscribe();

  while let Ok(event) = rx.recv().await {
    match event {
      RegistryEvent::LoadPlugins { username } => {
        if let Some(opts) = current_options() {
          let _ = PLUGINS.activate_for(username, opts.plugins);
        }
      }
      RegistryEvent::ControlModules { name, options, group } => {
        MODULE_MANAGER.control(name, options, group).await;
      }
      RegistryEvent::QuickTask { name } => {
        QUICK_TASK_MANAGER.execute(name);
      }
      RegistryEvent::DestroySwarm => {
        BOT_REGISTRY.destroy_swarm().await;
      }
      RegistryEvent::StopProcessing => {
        return;
      }
    }
  }
}

/// Вспомогательный макрос захвата
#[macro_export]
macro_rules! take_bot {
  ($username:expr, $func:expr) => {
    crate::service::core::registry::BOT_REGISTRY
      .async_get_bot($username, $func)
      .await;
  };
}
