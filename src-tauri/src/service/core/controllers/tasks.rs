use hashbrown::HashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::{sync::RwLock, task::JoinHandle};

use crate::api::emit::emit_log;

pub static TASKS: Lazy<Arc<TaskManager>> = Lazy::new(|| Arc::new(TaskManager::new()));

/// Название задачи бота
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TaskName {
  Spamming,
  Movement,
  Jumping,
  Shifting,
  Waving,
  AntiAfk,
  Stalker,
  Flight,
  Killaura,
  Scaffold,
  AntiFall,
  BowAim,
  Stealer,
  Miner,
  Farmer,
}

/// Структура задач бота
pub struct Tasks {
  pub root: HashMap<TaskName, JoinHandle<()>>,
  pub extra: HashMap<TaskName, Vec<JoinHandle<()>>>,
}

impl Tasks {
  /// Метод создания нового списка задач
  pub fn new() -> Self {
    Self {
      root: HashMap::new(),
      extra: HashMap::new(),
    }
  }

  /// Метод получения активности корневой задачи
  pub fn get_root_task_activity(&self, name: TaskName) -> bool {
    self.root.get(&name).is_some()
  }

  /// Метод добавления корневой задачи
  pub fn push_root_task(&mut self, name: TaskName, handle: JoinHandle<()>) {
    self.root.insert(name.clone(), handle);
  }

  /// Метод добавления дополнительной задачи
  pub fn push_extra_task(&mut self, name: TaskName, handle: JoinHandle<()>) {
    if let Some(tasks) = self.extra.get_mut(&name) {
      tasks.push(handle);
    } else {
      self.extra.insert(name, vec![handle]);
    }
  }

  /// Метод остановки корневой задачи и её дополнительных задач
  pub fn kill_task(&mut self, name: TaskName) {
    if let Some(tasks) = self.extra.get_mut(&name) {
      tasks.iter().for_each(|t| t.abort());
      tasks.clear();
    }

    if let Some(handle) = self.root.get(&name) {
      handle.abort();
    }

    self.extra.remove(&name);
    self.root.remove(&name);
  }

  /// Метод остановки всех задач
  pub fn kill_all_tasks(&mut self) {
    self
      .extra
      .iter()
      .for_each(|(_, tasks)| tasks.iter().for_each(|t| t.abort()));
    self.root.iter().for_each(|(_, t)| t.abort());
    self.extra.clear();
    self.root.clear();
  }
}

/// Менеджер управления задачами
pub struct TaskManager {
  pub map: RwLock<HashMap<String, Tasks>>,
}

impl TaskManager {
  /// Метод создания менеджера задач
  pub fn new() -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
    }
  }

  /// Метод регистрации бота
  pub fn register(&self, username: &str) {
    if let Ok(mut tasks) = self.map.try_write() {
      if let Some(t) = tasks.get_mut(username) {
        t.kill_all_tasks();
      }

      tasks.insert(username.to_string(), Tasks::new());
    }
  }

  /// Метод добавления корневой задачи указанному бота
  pub fn push_root_task_for(&self, username: &str, task_name: TaskName, root_task: JoinHandle<()>) {
    if let Ok(mut tasks) = self.map.try_write() {
      if let Some(t) = tasks.get_mut(username) {
        t.push_root_task(task_name, root_task);
      }
    }
  }

  /// Метод добавления дополнительной задачи указанному бота
  pub fn push_extra_task_for(&self, username: &str, task_name: TaskName, extra_task: JoinHandle<()>) {
    if let Ok(mut tasks) = self.map.try_write() {
      if let Some(t) = tasks.get_mut(username) {
        t.push_extra_task(task_name, extra_task);
      }
    }
  }

  /// Метод остановки определённой задачи указанного бота
  pub fn kill_task_for(&self, username: &str, task_name: TaskName) {
    if let Ok(mut tasks) = self.map.try_write() {
      if let Some(t) = tasks.get_mut(username) {
        t.kill_task(task_name);
      }
    }
  }

  /// Метод остановки всех задач указанного бота
  pub fn kill_all_tasks_for(&self, username: &str) {
    if let Ok(mut tasks) = self.map.try_write() {
      if let Some(t) = tasks.get_mut(username) {
        t.kill_all_tasks();
      }
    }
  }

  /// Метод удаления бота
  pub fn remove(&self, username: &str) {
    if let Ok(mut tasks) = self.map.try_write() {
      tasks.remove(username);
    }
  }

  /// Метод сброса задач указанного бота
  pub fn reset(&self, username: &str) {
    if let Ok(mut tasks) = self.map.try_write() {
      for (name, t) in tasks.iter_mut() {
        if name.as_str() == username {
          t.kill_all_tasks();
        }
      }
    }
  }

  /// Метод остановки всех активных задач и очистки хранилища
  pub async fn clear(&self) {
    let mut tasks = self.map.write().await;
    tasks.iter_mut().for_each(|(_, t)| t.kill_all_tasks());
    tasks.clear();

    emit_log("Активные задачи ботов остановлены", "extended");
  }

  /// Метод получения активности определённой задачи указанного бота
  pub fn get_task_activity(&self, username: &str, task_name: TaskName) -> bool {
    if let Ok(tasks) = self.map.try_read() {
      for (name, t) in tasks.iter() {
        if name.as_str() == username {
          return t.get_root_task_activity(task_name);
        }
      }
    }

    false
  }
}

/// Вспомогательная функция добавления корневой задачи указанному боту
pub fn push_root_task(username: &str, task_name: TaskName, root_task: JoinHandle<()>) {
  TASKS.push_root_task_for(username, task_name, root_task);
}

/// Вспомогательная функция добавления дополнительной задачи указанному боту
pub fn push_extra_task(username: &str, task_name: TaskName, extra_task: JoinHandle<()>) {
  TASKS.push_extra_task_for(username, task_name, extra_task);
}

/// Вспомогательная функция остановки определённой задачи указанного бота
pub fn kill_task(username: &str, task_name: TaskName) {
  TASKS.kill_task_for(username, task_name);
}
