use hashbrown::HashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::{sync::RwLock, task::JoinHandle};

use crate::api::emit::send_log;

pub static TASKS: Lazy<Arc<TaskManager>> = Lazy::new(|| Arc::new(TaskManager::new()));

/// Структура задач бота
pub struct Tasks {
  pub tasks: HashMap<String, Option<JoinHandle<()>>>,
  pub activity: HashMap<String, bool>,
}

impl Tasks {
  /// Метод создания нового списка задач
  pub fn new() -> Self {
    let names = vec![
      "spamming",
      "movement",
      "jumping",
      "shifting",
      "waving",
      "anti-afk",
      "stalker",
      "flight",
      "killaura",
      "scaffold",
      "anti-fall",
      "bow-aim",
      "stealer",
      "miner",
      "farmer",
    ];

    let mut tasks = HashMap::new();
    let mut activity = HashMap::new();

    for name in names {
      tasks.insert(name.to_string(), None);
      activity.insert(name.to_string(), false);
    }

    Self {
      tasks: tasks,
      activity: activity,
    }
  }

  /// Метод получения активности задачи
  pub fn get_task_activity(&self, name: &str) -> bool {
    if let Some(activity) = self.activity.get(name) {
      return *activity;
    }

    false
  }

  /// Метод добавления задачи
  pub fn push_task(&mut self, name: &str, handle: JoinHandle<()>) {
    self.tasks.insert(name.to_string(), Some(handle));
    self.activity.insert(name.to_string(), true);
  }

  /// Метод остановки задачи
  pub fn kill_task(&mut self, name: &str) {
    if let Some(handle) = self.tasks.get(name) {
      if let Some(h) = handle {
        h.abort();
        self.activity.insert(name.to_string(), false);
      }
    }
  }

  /// Метод остановки всех задач
  pub fn kill_all_tasks(&mut self) {
    for (name, handle) in self.tasks.iter() {
      if let Some(task) = handle {
        task.abort();
        self.activity.insert(name.clone(), false);
      }
    }
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
  pub fn push_task_for(&self, username: &str, task_name: &str, task_handle: JoinHandle<()>) {
    if let Ok(mut tasks) = self.map.try_write() {
      if let Some(t) = tasks.get_mut(username) {
        t.push_task(task_name, task_handle);
      }
    }
  }

  /// Метод остановки определённой задачи указанного бота
  pub fn kill_task_for(&self, username: &str, task_name: &str) {
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

    send_log(format!("Активные задачи ботов остановлены"), "extended");
  }

  /// Метод получения активности определённой задачи указанного бота
  pub fn get_task_activity(&self, username: &str, task: &str) -> bool {
    if let Ok(tasks) = self.map.try_read() {
      for (name, t) in tasks.iter() {
        if name.as_str() == username {
          return t.get_task_activity(task);
        }
      }
    }

    false
  }
}

/// Вспомогательная функция добавления задачи указанному боту
pub fn push_task(username: &str, task_name: &str, task_handle: JoinHandle<()>) {
  TASKS.push_task_for(username, task_name, task_handle);
}

/// Вспомогательная функция остановки определённой задачи указанного бота
pub fn kill_task(username: &str, task_name: &str) {
  TASKS.kill_task_for(username, task_name);
}
