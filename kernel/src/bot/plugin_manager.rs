use std::sync::LazyLock;

use hashbrown::HashMap;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::bot::plugins::*;
use crate::bot::traits::SalarixiPlugin;
use crate::launch::options::PluginOptions;
use crate::server::transfer::{emit_log, LogClass};

pub static PLUGINS: LazyLock<PluginManager> = LazyLock::new(|| PluginManager::new());

#[derive(Eq, Hash, PartialEq)]
pub enum PluginName {
  InstantArmorEquip,
  AutoTotem,
  AutoEat,
  PotionConsumer,
  AutoLook,
  AutoShield,
  AutoMending,
  PearlLeave,
}

pub struct PluginManager {
  tasks: RwLock<HashMap<u8, HashMap<PluginName, Option<JoinHandle<()>>>>>,
  plugins: PluginList,
}

struct PluginList {
  instant_armor_equip: InstantArmorEquipPlugin,
  auto_totem: AutoTotemPlugin,
  auto_eat: AutoEatPlugin,
  potion_consumer: PotionConsumerPlugin,
  auto_look: AutoLookPlugin,
  auto_shield: AutoShieldPlugin,
  auto_mending: AutoMendingPlugin,
  pearl_leave: PearlLeavePlugin,
}

impl PluginManager {
  pub fn new() -> Self {
    Self {
      tasks: RwLock::new(HashMap::new()),
      plugins: PluginList {
        instant_armor_equip: InstantArmorEquipPlugin::new(),
        auto_totem: AutoTotemPlugin::new(),
        auto_eat: AutoEatPlugin::new(),
        potion_consumer: PotionConsumerPlugin::new(),
        auto_look: AutoLookPlugin::new(),
        auto_shield: AutoShieldPlugin::new(),
        auto_mending: AutoMendingPlugin::new(),
        pearl_leave: PearlLeavePlugin::new(),
      },
    }
  }

  pub async fn activate_for(&'static self, index: u8, plugins: PluginOptions) {
    self.tasks.write().await.insert(index, HashMap::new());

    if plugins.instant_armor_equip.enabled {
      self
        .plugins
        .instant_armor_equip
        .activate(index, plugins.instant_armor_equip.clone());
    }

    if plugins.auto_totem.enabled {
      self.plugins.auto_totem.activate(index, plugins.auto_totem.clone());
    }

    if plugins.auto_eat.enabled {
      self.plugins.auto_eat.activate(index, plugins.auto_eat.clone());
    }

    if plugins.potion_consumer.enabled {
      self
        .plugins
        .potion_consumer
        .activate(index, plugins.potion_consumer.clone());
    }

    if plugins.auto_look.enabled {
      self.plugins.auto_look.activate(index, plugins.auto_look.clone());
    }

    if plugins.auto_shield.enabled {
      self.plugins.auto_shield.activate(index, plugins.auto_shield.clone());
    }

    if plugins.auto_mending.enabled {
      self.plugins.auto_mending.activate(index, plugins.auto_mending.clone());
    }

    if plugins.pearl_leave.enabled {
      self.plugins.pearl_leave.activate(index, plugins.pearl_leave.clone());
    }
  }

  pub fn push_task(&self, index: &u8, plugin_name: PluginName, task: JoinHandle<()>) {
    if let Ok(mut guard) = self.tasks.try_write() {
      if let Some(tasks) = guard.get_mut(index) {
        if let Some(old_task) = tasks.get(&plugin_name) {
          if let Some(active_old_task) = old_task {
            active_old_task.abort();
          }
        }

        tasks.insert(plugin_name, Some(task));
      }
    }
  }

  pub async fn kill_all_tasks_for(&self, index: &u8) {
    let mut map = self.tasks.write().await;

    if let Some(tasks) = map.get_mut(index) {
      for (_, t) in tasks {
        if let Some(h) = t {
          h.abort();
        }
      }
    }
  }

  pub async fn clear(&self) {
    let mut tasks = self.tasks.write().await;
    tasks.iter().for_each(|(_, m)| {
      m.iter().for_each(|(_, t)| {
        if let Some(h) = t {
          h.abort();
        }
      })
    });

    tasks.clear();
    tasks.shrink_to_fit();
    drop(tasks);

    emit_log("Активные задачи плагинов остановлены", LogClass::System);
  }
}
