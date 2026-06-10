use hashbrown::HashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::api::emit::send_log;
use crate::error;
use crate::service::core::bot::PROFILES;
use crate::service::core::launch::options::PluginOptions;
use crate::service::core::plugins::*;
use crate::service::core::traits::SalarixiPlugin;

pub static PLUGIN_MANAGER: Lazy<Arc<PluginManager>> = Lazy::new(|| Arc::new(PluginManager::new()));

/// Менеджер плагинов.
/// Изначально он хранит в себе объекты всех существующих плагинов и список их задач.
/// Данный менеджер выполняет роль загрузчика плагинов для бота.
pub struct PluginManager {
  tasks: RwLock<HashMap<String, HashMap<String, Option<JoinHandle<()>>>>>,
  plugins: PluginList,
}

struct PluginList {
  auto_armor: AutoArmorPlugin,
  auto_totem: AutoTotemPlugin,
  auto_eat: AutoEatPlugin,
  auto_potion: AutoPotionPlugin,
  auto_look: AutoLookPlugin,
  auto_shield: AutoShieldPlugin,
  auto_repair: AutoRepairPlugin,
}

impl PluginManager {
  pub fn new() -> Self {
    Self {
      tasks: RwLock::new(HashMap::new()),
      plugins: PluginList {
        auto_armor: AutoArmorPlugin::new(),
        auto_totem: AutoTotemPlugin::new(),
        auto_eat: AutoEatPlugin::new(),
        auto_potion: AutoPotionPlugin::new(),
        auto_look: AutoLookPlugin::new(),
        auto_shield: AutoShieldPlugin::new(),
        auto_repair: AutoRepairPlugin::new(),
      },
    }
  }

  pub fn activate_for(&'static self, username: String, plugins: PluginOptions) -> std::io::Result<()> {
    match self.tasks.try_write() {
      Ok(mut guard) => {
        guard.insert(username.clone(), HashMap::new());
      }
      Err(_) => {
        error!("plugin task list is already in use");
      }
    }

    if plugins.auto_armor {
      self.plugins.auto_armor.activate(username.clone())?;
    }

    if plugins.auto_totem {
      self.plugins.auto_totem.activate(username.clone())?;
    }

    if plugins.auto_eat {
      self.plugins.auto_eat.activate(username.clone())?;
    }

    if plugins.auto_potion {
      self.plugins.auto_potion.activate(username.clone())?;
    }

    if plugins.auto_look {
      self.plugins.auto_look.activate(username.clone())?;
    }

    if plugins.auto_shield {
      self.plugins.auto_shield.activate(username.clone())?;
    }

    if plugins.auto_repair {
      self.plugins.auto_repair.activate(username.clone())?;
    }

    Ok(())
  }

  pub fn push_task(&self, username: &str, plugin: &str, task: JoinHandle<()>) {
    if let Ok(mut guard) = self.tasks.try_write() {
      if let Some(tasks) = guard.get_mut(username) {
        if let Some(old_task) = tasks.get(plugin) {
          if let Some(active_old_task) = old_task {
            active_old_task.abort();
          }
        }

        tasks.insert(plugin.to_string(), Some(task));
      }
    }
  }

  pub fn destroy_all_tasks(&self, username: &str) {
    if let Ok(mut guard) = self.tasks.try_write() {
      if let Some(tasks) = guard.get_mut(username) {
        for (_, task) in tasks {
          if let Some(handle) = task {
            handle.abort();
          }
        }
      }
    }
  }

  pub fn clear(&self) {
    for (username, _) in PROFILES.get_all() {
      self.destroy_all_tasks(&username);
    }

    send_log(format!("Активные задачи плагинов остановлены"), "extended");
  }
}
