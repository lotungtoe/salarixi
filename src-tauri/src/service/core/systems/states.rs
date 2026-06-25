use hashbrown::HashMap;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::emit::emit_log;

pub static STATE_SYSTEM: Lazy<Arc<StateSystem>> = Lazy::new(|| Arc::new(StateSystem::new()));

/// Состояние бота
pub enum State {
  CanWalking,
  CanSprinting,
  CanEating,
  CanDrinking,
  CanAttacking,
  CanLooking,
  CanInteracting,
  IsWalking,
  IsSprinting,
  IsEating,
  IsDrinking,
  IsAttacking,
  IsLooking,
  IsInteracting,
}

/// Имя состояния бота
pub enum StateName {
  Walking,
  Sprinting,
  Eating,
  Drinking,
  Attacking,
  Looking,
  Interacting,
}

impl State {
  /// Метод конвертации строки в состояние
  pub fn from_str<'a>(str: impl Into<&'a str>) -> Option<Self> {
    match str.into() {
      "can_walking" => Some(Self::CanWalking),
      "can_sprinting" => Some(Self::CanSprinting),
      "can_eating" => Some(Self::CanEating),
      "can_drinking" => Some(Self::CanDrinking),
      "can_attacking" => Some(Self::CanAttacking),
      "can_looking" => Some(Self::CanLooking),
      "can_interacting" => Some(Self::CanInteracting),
      "is_walking" => Some(Self::IsWalking),
      "is_sprinting" => Some(Self::IsSprinting),
      "is_eating" => Some(Self::IsEating),
      "is_drinking" => Some(Self::IsDrinking),
      "is_attacking" => Some(Self::IsAttacking),
      "is_looking" => Some(Self::IsLooking),
      "is_interacting" => Some(Self::IsInteracting),
      _ => None,
    }
  }

  /// Метод получени индекса состояния
  pub fn index(&self) -> u16 {
    match self {
      Self::CanWalking => 0,
      Self::CanSprinting => 1,
      Self::CanEating => 2,
      Self::CanDrinking => 3,
      Self::CanAttacking => 4,
      Self::CanLooking => 5,
      Self::CanInteracting => 6,
      Self::IsWalking => 7,
      Self::IsSprinting => 8,
      Self::IsEating => 9,
      Self::IsDrinking => 10,
      Self::IsAttacking => 11,
      Self::IsLooking => 12,
      Self::IsInteracting => 13,
    }
  }
}

/// Структура состояний бота
pub struct States {
  pub mask: AtomicU16,
}

impl States {
  /// Метод создания состояний по умолчанию
  pub fn new() -> Self {
    Self {
      mask: AtomicU16::new(0x7F),
    }
  }

  /// Метод получения значения указанного состояния
  pub fn get(&self, state: State) -> bool {
    let mask = self.mask.load(Ordering::SeqCst);
    (mask >> state.index()) & 1 == 1
  }

  /// Метод установки значения для указанного состояния
  pub fn set(&mut self, state: State, value: bool) {
    let index = state.index();

    loop {
      let mask = self.mask.load(Ordering::SeqCst);
      let new_mask = if value {
        mask | (1 << index)
      } else {
        mask & !(1 << index)
      };

      match self
        .mask
        .compare_exchange(mask, new_mask, Ordering::SeqCst, Ordering::SeqCst)
      {
        Ok(_) => break,
        Err(_) => continue,
      }
    }
  }
}

/// Система управления состояниями ботов
pub struct StateSystem {
  pub map: RwLock<HashMap<String, States>>,
}

impl StateSystem {
  /// Метод создания менеджера состояний
  pub fn new() -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
    }
  }

  /// Метод регистрации бота
  pub async fn register(&self, username: &str) {
    self.map.write().await.insert(username.to_string(), States::new());
  }

  /// Метод очистки хранилища состояний
  pub async fn clear(&self) {
    self.map.write().await.clear();
    emit_log("Состояния ботов очищены", "extended");
  }

  /// Метод сброса состояний указанного бота
  pub async fn reset(&self, username: &str) {
    let mut guard = self.map.write().await;
    if let Some(s) = guard.get_mut(username) {
      s.mask.store(0x7F, Ordering::SeqCst);
    }
  }

  /// Метод удаления всех состояний указанного бота
  pub async fn remove(&self, username: &str) {
    self.map.write().await.remove(username);
  }
}

/// Вспомогательная функция получения состояния
pub async fn getst(username: &str, state: State) -> bool {
  let guard = STATE_SYSTEM.map.read().await;
  if let Some(s) = guard.get(username) {
    return s.get(state);
  }

  false
}

/// Вспомогательная функция установки значения для состояния
pub async fn setst(username: &str, state: State, value: bool) {
  let mut guard = STATE_SYSTEM.map.write().await;
  if let Some(s) = guard.get_mut(username) {
    s.set(state, value);
  }
}

/// Вспомогательная функция взаимной установки значений для ```can``` и ```is``` состояний
pub async fn setmst(username: &str, state_name: StateName, current_value: bool) {
  let (is_state, can_state) = match state_name {
    StateName::Walking => (State::IsWalking, State::CanWalking),
    StateName::Sprinting => (State::IsSprinting, State::CanSprinting),
    StateName::Eating => (State::IsEating, State::CanEating),
    StateName::Drinking => (State::IsDrinking, State::CanDrinking),
    StateName::Attacking => (State::IsAttacking, State::CanAttacking),
    StateName::Looking => (State::IsLooking, State::CanLooking),
    StateName::Interacting => (State::IsInteracting, State::CanInteracting),
  };

  let mut guard = STATE_SYSTEM.map.write().await;
  if let Some(s) = guard.get_mut(username) {
    s.set(is_state, current_value);
    s.set(can_state, !current_value);
  }
}

/// Вспомогательная функция попытки получения состояния
pub fn try_getst(username: &str, state: State) -> bool {
  if let Ok(states) = STATE_SYSTEM.map.try_read() {
    if let Some(s) = states.get(username) {
      return s.get(state);
    }
  }

  false
}

/// Вспомогательная функция попытки установки значения для состояния
pub fn try_setst(username: &str, state: State, value: bool) {
  if let Ok(mut states) = STATE_SYSTEM.map.try_write() {
    if let Some(s) = states.get_mut(username) {
      s.set(state, value);
    }
  }
}
