use hashbrown::HashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::emit::send_log;

pub static STATES: Lazy<Arc<StateManager>> = Lazy::new(|| Arc::new(StateManager::new()));

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

impl State {
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
}

pub struct States {
  pub can_walking: bool,
  pub can_sprinting: bool,
  pub can_eating: bool,
  pub can_drinking: bool,
  pub can_attacking: bool,
  pub can_looking: bool,
  pub can_interacting: bool,
  pub is_walking: bool,
  pub is_sprinting: bool,
  pub is_eating: bool,
  pub is_drinking: bool,
  pub is_attacking: bool,
  pub is_looking: bool,
  pub is_interacting: bool,
}

impl States {
  pub fn new() -> Self {
    Self {
      can_walking: true,
      can_sprinting: true,
      can_eating: true,
      can_drinking: true,
      can_attacking: true,
      can_looking: true,
      can_interacting: true,
      is_walking: false,
      is_sprinting: false,
      is_eating: false,
      is_drinking: false,
      is_attacking: false,
      is_looking: false,
      is_interacting: false,
    }
  }

  pub fn get(&self, state: State) -> bool {
    match state {
      State::CanWalking => self.can_walking,
      State::CanSprinting => self.can_sprinting,
      State::CanEating => self.can_eating,
      State::CanDrinking => self.can_drinking,
      State::CanAttacking => self.can_attacking,
      State::CanLooking => self.can_looking,
      State::CanInteracting => self.can_interacting,
      State::IsWalking => self.is_walking,
      State::IsSprinting => self.is_sprinting,
      State::IsEating => self.is_eating,
      State::IsDrinking => self.is_drinking,
      State::IsAttacking => self.is_attacking,
      State::IsLooking => self.is_looking,
      State::IsInteracting => self.is_interacting,
    }
  }

  pub fn set(&mut self, state: State, value: bool) {
    match state {
      State::CanWalking => {
        self.can_walking = value;
      }
      State::CanSprinting => {
        self.can_sprinting = value;
      }
      State::CanEating => {
        self.can_eating = value;
      }
      State::CanDrinking => {
        self.can_drinking = value;
      }
      State::CanAttacking => {
        self.can_attacking = value;
      }
      State::CanLooking => {
        self.can_looking = value;
      }
      State::CanInteracting => {
        self.can_interacting = value;
      }
      State::IsWalking => {
        self.is_walking = value;
      }
      State::IsSprinting => {
        self.is_sprinting = value;
      }
      State::IsEating => {
        self.is_eating = value;
      }
      State::IsDrinking => {
        self.is_drinking = value;
      }
      State::IsAttacking => {
        self.is_attacking = value;
      }
      State::IsLooking => {
        self.is_looking = value;
      }
      State::IsInteracting => {
        self.is_interacting = value;
      }
    }
  }
}

pub struct StateManager {
  pub map: RwLock<HashMap<String, States>>,
}

impl StateManager {
  pub fn new() -> Self {
    Self {
      map: RwLock::new(HashMap::new()),
    }
  }

  pub fn push(&self, username: &str) {
    if let Ok(mut states) = self.map.try_write() {
      if !states.contains_key(username) {
        states.insert(username.to_string(), States::new());
      }
    }
  }

  pub fn clear(&self) {
    if let Ok(mut states) = self.map.try_write() {
      states.clear();
    }

    send_log(format!("Состояния ботов очищены"), "extended");
  }

  pub fn reset(&self, username: &str) {
    if let Ok(mut states) = self.map.try_write() {
      if let Some(s) = states.get_mut(username) {
        s.can_walking = true;
        s.can_sprinting = true;
        s.can_eating = true;
        s.can_drinking = true;
        s.can_attacking = true;
        s.can_looking = true;
        s.can_interacting = true;
        s.is_walking = false;
        s.is_sprinting = false;
        s.is_eating = false;
        s.is_drinking = false;
        s.is_attacking = false;
        s.is_looking = false;
        s.is_interacting = false;
      }
    }
  }
}

/// Вспомогательная функция получения состояния
pub fn get_state(username: &str, state: State) -> bool {
  if let Ok(states) = STATES.map.try_read() {
    if let Some(s) = states.get(username) {
      return s.get(state);
    }
  }

  false
}

/// Вспомогательная функция установки значения для состояния
pub fn set_state(username: &str, state: State, value: bool) {
  if let Ok(mut states) = STATES.map.try_write() {
    if let Some(s) = states.get_mut(username) {
      s.set(state, value);
    }
  }
}

/// Вспомогательная функция взаимной установки значений для ```can``` и ```is``` состояний
pub fn set_mutual_states(username: &str, name: &str, current_value: bool) {
  let (is_state, can_state) = match name {
    "walking" => (State::IsWalking, State::CanWalking),
    "sprinting" => (State::IsSprinting, State::CanSprinting),
    "eating" => (State::IsEating, State::CanEating),
    "drinking" => (State::IsDrinking, State::CanDrinking),
    "attacking" => (State::IsAttacking, State::CanAttacking),
    "looking" => (State::IsLooking, State::CanLooking),
    "interacting" => (State::IsInteracting, State::CanInteracting),
    _ => return,
  };

  if let Ok(mut states) = STATES.map.try_write() {
    if let Some(s) = states.get_mut(username) {
      s.set(is_state, current_value);
      s.set(can_state, !current_value);
    }
  }
}
