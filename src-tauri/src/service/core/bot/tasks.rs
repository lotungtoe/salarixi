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
  pub tasks: HashMap<TaskName, JoinHandle<()>>,
}

impl Tasks {
  /// Метод создания нового списка задач
  pub fn new() -> Self {
    Self { tasks: HashMap::new() }
  }

  /// Метод получения активности задачи
  pub fn get_task_activity(&self, name: TaskName) -> bool {
    self.tasks.get(&name).is_some()
  }

  /// Метод добавления задачи
  pub fn push_task(&mut self, name: TaskName, handle: JoinHandle<()>) {
    self.tasks.insert(name.clone(), handle);
  }

  /// Метод остановки задачи
  pub fn kill_task(&mut self, name: TaskName) {
    if let Some(handle) = self.tasks.get(&name) {
      handle.abort();
      self.tasks.remove(&name);
    }
  }

  /// Метод остановки всех задач
  pub fn kill_all_tasks(&mut self) {
    self.tasks.iter().for_each(|(_, handle)| handle.abort());
    self.tasks.clear();
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

  /// Метод добавления задачи указанному бота
  pub fn push_task_for(&self, username: &str, task_name: TaskName, task_handle: JoinHandle<()>) {
    if let Ok(mut tasks) = self.map.try_write() {
      if let Some(t) = tasks.get_mut(username) {
        t.push_task(task_name, task_handle);
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
  pub fn clear(&self) {
    if let Ok(mut tasks) = self.map.try_write() {
      tasks.iter_mut().for_each(|(_, t)| t.kill_all_tasks());
      tasks.clear();
    }

    emit_log("Активные задачи ботов остановлены", "extended");
  }

  /// Метод получения активности определённой задачи указанного бота
  pub fn get_task_activity(&self, username: &str, task_name: TaskName) -> bool {
    if let Ok(tasks) = self.map.try_read() {
      for (name, t) in tasks.iter() {
        if name.as_str() == username {
          return t.get_task_activity(task_name);
        }
      }
    }

    false
  }
}

/// Вспомогательная функция добавления задачи указанному боту
pub fn push_task(username: &str, task_name: TaskName, task_handle: JoinHandle<()>) {
  TASKS.push_task_for(username, task_name, task_handle);
}

/// Вспомогательная функция остановки определённой задачи указанного бота
pub fn kill_task(username: &str, task_name: TaskName) {
  TASKS.kill_task_for(username, task_name);
}
