use hashbrown::HashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

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
  pub fn pushrtsk(&mut self, name: TaskName, handle: JoinHandle<()>) {
    self.root.insert(name.clone(), handle);
  }

  /// Метод добавления дополнительной задачи
  pub fn pushetsk(&mut self, name: TaskName, handle: JoinHandle<()>) {
    if let Some(tasks) = self.extra.get_mut(&name) {
      tasks.push(handle);
    } else {
      self.extra.insert(name, vec![handle]);
    }
  }

  /// Метод остановки корневой задачи и её дополнительных задач
  pub fn killtsk(&mut self, name: TaskName) {
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
  pub async fn register(&self, username: &str) {
    let mut guard = self.map.write().await;
    if let Some(t) = guard.get_mut(username) {
      t.kill_all_tasks();
    }

    guard.insert(username.to_string(), Tasks::new());
  }

  /// Метод остановки всех задач указанного бота
  pub async fn kill_all_tasks_for(&self, username: &str) {
    let mut guard = self.map.write().await;
    if let Some(t) = guard.get_mut(username) {
      t.kill_all_tasks();
    }
  }

  /// Метод удаления бота
  pub async fn remove(&self, username: &str) {
    self.map.write().await.remove(username);
  }

  /// Метод остановки всех активных задач и очистки хранилища
  pub async fn clear(&self) {
    let mut guard = self.map.write().await;
    guard.iter_mut().for_each(|(_, t)| t.kill_all_tasks());
    guard.clear();

    emit_log("Активные задачи ботов остановлены", "extended");
  }
}

/// Вспомогательная функция получения активности определённой задачи указанного бота
pub async fn gettskact(username: &str, task_name: TaskName) -> bool {
  let guard = TASKS.map.read().await;

  for (name, t) in guard.iter() {
    if name.as_str() == username {
      return t.get_root_task_activity(task_name);
    }
  }

  false
}

/// Вспомогательная функция добавления корневой задачи указанному бота
pub async fn pushrtsk(username: &str, task_name: TaskName, root_task: JoinHandle<()>) {
  let mut guard = TASKS.map.write().await;
  if let Some(t) = guard.get_mut(username) {
    t.pushrtsk(task_name, root_task);
  }
}

/// Вспомогательная функция добавления дополнительной задачи указанному бота
pub async fn pushetsk(username: &str, task_name: TaskName, extra_task: JoinHandle<()>) {
  let mut guard = TASKS.map.write().await;
  if let Some(t) = guard.get_mut(username) {
    t.pushetsk(task_name, extra_task);
  }
}

/// Вспомогательная функция остановки определённой задачи указанного бота
pub async fn killtsk(username: &str, task_name: TaskName) {
  let mut guard = TASKS.map.write().await;
  if let Some(t) = guard.get_mut(username) {
    t.killtsk(task_name);
  }
}
