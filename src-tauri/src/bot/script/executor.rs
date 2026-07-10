use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};

use azalea::bot::BotClientExt;
use azalea::prelude::PathfinderClientExt;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::{BlockPos, SprintDirection, Vec3, WalkDirection};
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use rhai::{Dynamic, Engine, EvalAltResult, Map};
use tokio::sync::RwLock;

use crate::api::emit::{emit_log, emit_msg};
use crate::bot::common::convert_hotbar_slot_to_inventory_slot;
use crate::bot::extensions::{
  go_to, BotDefaultExt, BotInteractExt, BotInventoryExt, BotMovementExt, BotPhysicsExt, BotRotationExt, ClickMode,
  EntityType,
};
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::bot::systems::registry::REGISTRY_SYSTEM;
use crate::bot::systems::states::{try_getst, try_setst, State};
use crate::launch::process::{current_options, process_is_active};
use crate::other::webhook::*;

pub static SCRIPT_EXECUTOR: Lazy<ScriptExecutor> = Lazy::new(|| ScriptExecutor::new());

static EXECUTOR_STATE_SYSTEM: Lazy<RwLock<HashMap<String, ()>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static EXECUTOR_GLOBAL_STATE: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Функция генерации кода
fn generate_code<'a>(su: &'a str, mu: &'a str) -> i16 {
  // По логике здесь не может быть ошибки, ибо .write() у Index Map берётся только
  // во время добавления аккаунтов и после остановки ботов. В эти моменты сценарий
  // не может выполнятся. Это не самая лучшая реализация но и не костыль.
  let index_map = match INDEX_SYSTEM.map.try_read() {
    Ok(m) => m,
    Err(_) => return 0,
  };

  let tu = if su.is_empty() {
    if mu == "*" {
      return -1;
    }

    mu
  } else {
    su
  };

  for (i, u) in index_map.iter().enumerate() {
    if u == tu {
      return i as i16;
    }
  }

  0
}

/// Функция создания динамичного вектора координат
fn create_dynamic_vec(x: f64, y: f64, z: f64) -> Dynamic {
  let mut map = Map::new();
  map.insert("x".into(), Dynamic::from_float(x));
  map.insert("y".into(), Dynamic::from_float(y));
  map.insert("z".into(), Dynamic::from_float(z));
  Dynamic::from_map(map)
}

/// Функция создания динамичного профиля
fn create_dynamic_profile(index: &u8) -> Dynamic {
  let mut map = Map::new();

  if let Some(profile) = PROFILE_SYSTEM.try_get(index) {
    map.insert("ping".into(), Dynamic::from_int(profile.ping as i64));
    map.insert("captcha_caught".into(), Dynamic::from_bool(profile.captcha_caught));
    map.insert("registered".into(), Dynamic::from_bool(profile.registered));
    map.insert("logined".into(), Dynamic::from_bool(profile.logined));
    map.insert("group".into(), Dynamic::from_str(profile.group.as_str()).unwrap());
    map.insert("health".into(), Dynamic::from_int(profile.health as i64));

    if let Some(password) = profile.password {
      map.insert("password".into(), Dynamic::from_str(password.as_str()).unwrap());
    }

    if let Some(email) = profile.email {
      map.insert("email".into(), Dynamic::from_str(email.as_str()).unwrap());
    }
  }

  Dynamic::from_map(map)
}

pub struct ScriptExecutor;

impl ScriptExecutor {
  pub fn new() -> Self {
    Self
  }

  fn chat(code: i16, message: String) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.chat(&message);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.chat(&message);
      }
    }
  }

  fn jump(code: i16) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.jump();
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.jump();
      }
    }
  }

  fn swing_arm(code: i16) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.swing_arm();
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.swing_arm();
      }
    }
  }

  fn set_jumping(code: i16, state: bool) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.set_jumping(state);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.set_jumping(state);
      }
    }
  }

  fn set_crouching(code: i16, state: bool) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.set_crouching(state);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.set_crouching(state);
      }
    }
  }

  fn set_velocity(code: i16, axis: &str, velocity: f64) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.set_velocity(axis, velocity);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.set_velocity(axis, velocity);
      }
    }
  }

  fn set_on_ground(code: i16, on_ground: bool) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.set_on_ground(on_ground);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.set_on_ground(on_ground);
      }
    }
  }

  fn set_old_position(code: i16, x: f64, y: f64, z: f64) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.set_old_position(x, y, z);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.set_old_position(x, y, z);
      }
    }
  }

  fn set_no_jump_delay(code: i16, delay: u32) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.set_no_jump_delay(delay);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.set_no_jump_delay(delay);
      }
    }
  }

  fn set_was_touching_water(code: i16, state: bool) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.set_was_touching_water(state);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.set_was_touching_water(state);
      }
    }
  }

  fn set_has_impulse(code: i16, state: bool) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.set_has_impulse(state);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.set_has_impulse(state);
      }
    }
  }

  fn start_walking(code: i16, direction_name: &str) {
    let direction = match direction_name {
      "forward" => WalkDirection::Forward,
      "backward" => WalkDirection::Backward,
      "left" => WalkDirection::Left,
      "right" => WalkDirection::Right,
      "forward-left" => WalkDirection::ForwardLeft,
      "forward-right" => WalkDirection::ForwardRight,
      "backward-left" => WalkDirection::BackwardLeft,
      "backward-right" => WalkDirection::BackwardRight,
      _ => return,
    };

    tokio::spawn(async move {
      if code == -1 {
        for index in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
            bot.start_walking(index, direction).await;
          }
        }
      } else {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
          bot.start_walking(&(code as u8), direction).await;
        }
      }
    });
  }

  fn start_sprinting(code: i16, direction_name: &str) {
    let direction = match direction_name {
      "forward" => SprintDirection::Forward,
      "forward-left" => SprintDirection::ForwardLeft,
      "forward-right" => SprintDirection::ForwardRight,
      _ => return,
    };

    tokio::spawn(async move {
      if code == -1 {
        for index in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
            bot.start_sprinting(index, direction).await;
          }
        }
      } else {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
          bot.start_sprinting(&(code as u8), direction).await;
        }
      }
    });
  }

  fn start_pathfinder(code: i16, x: i32, z: i32) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        go_to(*index, x, z);
      });
    } else {
      go_to(code as u8, x, z);
    }
  }

  fn stop_move(code: i16) {
    tokio::spawn(async move {
      if code == -1 {
        for index in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
            bot.stop_move(index).await;
          }
        }
      } else {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
          bot.stop_move(&(code as u8)).await;
        }
      }
    });
  }

  fn stop_pathfinder(code: i16) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.stop_pathfinding();
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.stop_pathfinding();
      }
    }
  }

  fn look_at(code: i16, x: f64, y: f64, z: f64) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.look_at(Vec3 { x, y, z });
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.look_at(Vec3 { x, y, z });
      }
    }
  }

  fn look_at_block(code: i16, x: i32, y: i32, z: i32) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.look_at(BlockPos::new(x, y, z).center());
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.look_at(BlockPos::new(x, y, z).center());
      }
    }
  }

  fn place_block(code: i16, x: i32, y: i32, z: i32) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.block_interact(BlockPos::new(x, y, z));
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.block_interact(BlockPos::new(x, y, z));
      }
    }
  }

  fn drop_selected_item(code: i16, lock: bool) {
    tokio::spawn(async move {
      if code == -1 {
        for index in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
            let selected_slot = convert_hotbar_slot_to_inventory_slot(bot.get_selected_slot());
            bot
              .inventory_click(index, selected_slot, ClickMode::DropAll, lock)
              .await;
          }
        }
      } else {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
          let selected_slot = convert_hotbar_slot_to_inventory_slot(bot.get_selected_slot());
          bot
            .inventory_click(&(code as u8), selected_slot, ClickMode::DropAll, lock)
            .await;
        }
      }
    });
  }

  fn drop_all_items(code: i16, lock: bool) {
    tokio::spawn(async move {
      if code == -1 {
        for index in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
            if let Some(menu) = bot.get_inventory_menu() {
              for (slot, _) in menu.slots().iter().enumerate() {
                bot.inventory_click(index, slot, ClickMode::DropAll, lock).await;
              }
            }
          }
        }
      } else {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
          if let Some(menu) = bot.get_inventory_menu() {
            for (slot, _) in menu.slots().iter().enumerate() {
              bot.inventory_click(&(code as u8), slot, ClickMode::DropAll, lock).await;
            }
          }
        }
      }
    });
  }

  fn inv_click(code: i16, slot: usize, mode: &str, lock: bool) {
    let mode_string = mode.to_string();

    tokio::spawn(async move {
      if code == -1 {
        for index in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
            bot
              .inventory_click(index, slot, ClickMode::from(mode_string.as_str()), lock)
              .await;
          }
        }
      } else {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
          bot
            .inventory_click(&(code as u8), slot, ClickMode::from(mode_string.as_str()), lock)
            .await;
        }
      }
    });
  }

  fn inv_click_on(code: i16, name: &str, mode: &str, lock: bool) {
    let name_string = name.to_string();
    let mode_string = mode.to_string();

    tokio::spawn(async move {
      if code == -1 {
        for index in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
            bot
              .inventory_click_on(index, &name_string, ClickMode::from(mode_string.as_str()), lock)
              .await;
          }
        }
      } else {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
          bot
            .inventory_click_on(&(code as u8), &name_string, ClickMode::from(mode_string.as_str()), lock)
            .await;
        }
      }
    });
  }

  fn set_selected_slot(code: i16, slot: u8) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          bot.set_selected_hotbar_slot(slot);
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        bot.set_selected_hotbar_slot(slot);
      }
    }
  }

  fn attack(code: i16, target: String, distance: f64, look_at_target: bool) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        if let Some(bot) = REGISTRY_SYSTEM.get_bot(index) {
          if let Some(entity) = bot.find_nearest_entity(EntityType::from(target.clone()), distance) {
            if look_at_target {
              bot.look_at_entity(entity, false);
            }

            bot.attack(entity);
          }
        }
      });
    } else {
      if let Some(bot) = REGISTRY_SYSTEM.get_bot(&(code as u8)) {
        if let Some(entity) = bot.find_nearest_entity(EntityType::from(target), distance) {
          if look_at_target {
            bot.look_at_entity(entity, false);
          }

          bot.attack(entity);
        }
      }
    }
  }

  fn start_use_item(code: i16, hand: &str) {
    let interaction_hand = if hand == "offhand" {
      InteractionHand::OffHand
    } else {
      InteractionHand::MainHand
    };

    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        REGISTRY_SYSTEM.get_bot(index).map(|bot| {
          bot.start_use_held_item(interaction_hand);
        });
      });
    } else {
      REGISTRY_SYSTEM.get_bot(&(code as u8)).map(|bot| {
        bot.start_use_held_item(interaction_hand);
      });
    }
  }

  fn release_use_item(code: i16) {
    if code == -1 {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|index| {
        REGISTRY_SYSTEM.get_bot(index).map(|bot| {
          bot.release_use_held_item();
        });
      });
    } else {
      REGISTRY_SYSTEM.get_bot(&(code as u8)).map(|bot| {
        bot.release_use_held_item();
      });
    }
  }

  /// Метод исполнения пользовательского сценария
  pub fn execute(&self, su: String, si: u8, script: String) {
    let mut states = match EXECUTOR_STATE_SYSTEM.try_write() {
      Ok(g) => g,
      Err(_) => {
        if su.is_empty() {
          emit_msg("Сценарий", "Не удалось выполнить сценарий");
          emit_log(
            "Не выполнить запустить сценарий: States are currently occupied",
            "warning",
          );
        } else {
          emit_msg("Сценарий", format!("Не удалось выполнить сценарий для {}", su));
          emit_log(
            format!(
              "Не выполнить запустить сценарий для {}: States are currently occupied",
              su
            ),
            "warning",
          );
        }

        return;
      }
    };

    if su.is_empty() {
      if !states.is_empty() || EXECUTOR_GLOBAL_STATE.load(Ordering::SeqCst) {
        emit_msg("Сценарий", "Не удалось выполнить сценарий");
        emit_log("Не выполнить запустить сценарий: Script is already running", "warning");
        return;
      }
    } else {
      if states.contains_key(&su) {
        emit_msg("Сценарий", format!("Не удалось выполнить сценарий для {}", su));
        emit_log(
          format!("Не выполнить запустить сценарий для {}: Script is already running", su),
          "warning",
        );
        return;
      }
    }

    if su.is_empty() {
      EXECUTOR_GLOBAL_STATE.store(true, Ordering::SeqCst);
    } else {
      states.insert(su.clone(), ());
    }

    drop(states);

    let mut engine = Engine::new();

    let su_clone = su.clone();
    engine.on_progress(move |_| {
      if !process_is_active() {
        return Some(Dynamic::UNIT);
      }

      if su_clone.is_empty() {
        if !EXECUTOR_GLOBAL_STATE.load(Ordering::SeqCst) {
          return Some(Dynamic::UNIT);
        }
      } else {
        if let Ok(states) = EXECUTOR_STATE_SYSTEM.try_read() {
          if !states.contains_key(&su_clone) {
            return Some(Dynamic::UNIT);
          }
        }
      }

      None
    });

    engine
      .register_fn("println", |str: &str| {
        println!("[ salarixi / script ]: {}", str);
      })
      .register_fn("log", |text: &str, class: &str| {
        emit_log(text.to_string(), class);
      })
      .register_fn("message", |name: &str, content: &str| {
        emit_msg(name, content.to_string());
      })
      .register_fn("webhook", |content: &str| {
        let content_string = content.to_string();
        tokio::spawn(async move {
          if let Some(opts) = current_options().await {
            send_webhook(opts.webhook.url, content_string);
          }
        });
      });

    engine.register_fn("active_bots_count", || PROFILE_SYSTEM.try_get_connected_count() as i64);

    engine
      .register_fn("get_bot_id", {
        let su = su.clone();
        move |mu: &str| {
          if mu == "*" {
            return 0;
          }

          REGISTRY_SYSTEM
            .get_bot(&(generate_code(&su, mu) as u8))
            .map(|b| b.id().0 as i64)
            .unwrap_or(0)
        }
      })
      .register_fn("get_bot_ping", {
        let su = su.clone();
        move |mu: &str| {
          if mu == "*" {
            return 0;
          }

          REGISTRY_SYSTEM
            .get_bot(&(generate_code(&su, mu) as u8))
            .map(|b| b.ping() as i64)
            .unwrap_or(0)
        }
      })
      .register_fn("get_bot_feet_pos", {
        let su = su.clone();
        move |mu: &str| {
          if mu == "*" {
            return create_dynamic_vec(0.0, 0.0, 0.0);
          }

          REGISTRY_SYSTEM
            .get_bot(&(generate_code(&su, mu) as u8))
            .map(|b| {
              let pos = b.feet_pos();
              create_dynamic_vec(pos.x, pos.y, pos.z)
            })
            .unwrap_or(create_dynamic_vec(0.0, 0.0, 0.0))
        }
      })
      .register_fn("get_bot_eye_pos", {
        let su = su.clone();
        move |mu: &str| {
          if mu == "*" {
            return create_dynamic_vec(0.0, 0.0, 0.0);
          }

          REGISTRY_SYSTEM
            .get_bot(&(generate_code(&su, mu) as u8))
            .map(|b| {
              let pos = b.eye_pos();
              create_dynamic_vec(pos.x, pos.y, pos.z)
            })
            .unwrap_or(create_dynamic_vec(0.0, 0.0, 0.0))
        }
      })
      .register_fn("get_bot_health", {
        let su = su.clone();
        move |mu: &str| {
          if mu == "*" {
            return 0.0;
          }

          REGISTRY_SYSTEM
            .get_bot(&(generate_code(&su, mu) as u8))
            .map(|b| b.get_health() as f64)
            .unwrap_or(0.0)
        }
      })
      .register_fn("get_bot_satiety", {
        let su = su.clone();
        move |mu: &str| {
          if mu == "*" {
            return 0;
          }

          REGISTRY_SYSTEM
            .get_bot(&(generate_code(&su, mu) as u8))
            .map(|b| b.get_satiety() as i64)
            .unwrap_or(0)
        }
      })
      .register_fn("bot_is_workable", {
        let su = su.clone();
        move |mu: &str| {
          if mu == "*" {
            return false;
          }

          REGISTRY_SYSTEM
            .get_bot(&(generate_code(&su, mu) as u8))
            .map(|b| b.workable())
            .unwrap_or(false)
        }
      })
      .register_fn("get_bot_profile", {
        let su = su.clone();
        move |mu: &str| {
          if mu == "*" {
            return Dynamic::ZERO;
          }

          create_dynamic_profile(&(generate_code(&su, mu) as u8))
        }
      });

    if !su.is_empty() {
      engine
        .register_fn("current_bot_id", move || {
          REGISTRY_SYSTEM.get_bot(&si).map(|b| b.id().0 as i64).unwrap_or(0)
        })
        .register_fn("current_bot_ping", move || {
          REGISTRY_SYSTEM.get_bot(&si).map(|b| b.ping() as i64).unwrap_or(0)
        })
        .register_fn("current_bot_feet_pos", move || {
          REGISTRY_SYSTEM
            .get_bot(&si)
            .map(|b| {
              let pos = b.feet_pos();
              create_dynamic_vec(pos.x, pos.y, pos.z)
            })
            .unwrap_or(create_dynamic_vec(0.0, 0.0, 0.0))
        })
        .register_fn("current_bot_eye_pos", move || {
          REGISTRY_SYSTEM
            .get_bot(&si)
            .map(|b| {
              let pos = b.eye_pos();
              create_dynamic_vec(pos.x, pos.y, pos.z)
            })
            .unwrap_or(create_dynamic_vec(0.0, 0.0, 0.0))
        })
        .register_fn("current_bot_health", move || {
          REGISTRY_SYSTEM
            .get_bot(&si)
            .map(|b| b.get_health() as f64)
            .unwrap_or(0.0)
        })
        .register_fn("current_bot_satiety", move || {
          REGISTRY_SYSTEM
            .get_bot(&si)
            .map(|b| b.get_satiety() as i64)
            .unwrap_or(0)
        })
        .register_fn("current_bot_profile", move || create_dynamic_profile(&si));
    }

    engine
      .register_fn("chat", {
        let su = su.clone();
        move |mu: &str, message: &str| Self::chat(generate_code(&su, mu), message.to_string())
      })
      .register_fn("jump", {
        let su = su.clone();
        move |mu: &str| Self::jump(generate_code(&su, mu))
      })
      .register_fn("swing_arm", {
        let su = su.clone();
        move |mu: &str| Self::swing_arm(generate_code(&su, mu))
      })
      .register_fn("set_jumping", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_jumping(generate_code(&su, mu), state)
      })
      .register_fn("set_crouching", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_crouching(generate_code(&su, mu), state)
      })
      .register_fn("set_velocity", {
        let su = su.clone();
        move |mu: &str, axis: &str, velocity: f64| Self::set_velocity(generate_code(&su, mu), axis, velocity)
      })
      .register_fn("set_on_ground", {
        let su = su.clone();
        move |mu: &str, on_ground: bool| Self::set_on_ground(generate_code(&su, mu), on_ground)
      })
      .register_fn("set_old_position", {
        let su = su.clone();
        move |mu: &str, x: f64, y: f64, z: f64| Self::set_old_position(generate_code(&su, mu), x, y, z)
      })
      .register_fn("set_no_jump_delay", {
        let su = su.clone();
        move |mu: &str, delay: i64| Self::set_no_jump_delay(generate_code(&su, mu), delay as u32)
      })
      .register_fn("set_was_touching_water", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_was_touching_water(generate_code(&su, mu), state)
      })
      .register_fn("set_has_impulse", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_has_impulse(generate_code(&su, mu), state)
      })
      .register_fn("start_walking", {
        let su = su.clone();
        move |mu: &str, direction: &str| Self::start_walking(generate_code(&su, mu), direction)
      })
      .register_fn("start_sprinting", {
        let su = su.clone();
        move |mu: &str, direction: &str| Self::start_sprinting(generate_code(&su, mu), direction)
      })
      .register_fn("start_pathfinder", {
        let su = su.clone();
        move |mu: &str, x: i64, z: i64| Self::start_pathfinder(generate_code(&su, mu), x as i32, z as i32)
      })
      .register_fn("stop_move", {
        let su = su.clone();
        move |mu: &str| Self::stop_move(generate_code(&su, mu))
      })
      .register_fn("stop_pathfinder", {
        let su = su.clone();
        move |mu: &str| Self::stop_pathfinder(generate_code(&su, mu))
      });

    engine
      .register_fn("place_block", {
        let su = su.clone();
        move |mu: &str, x: i64, y: i64, z: i64| Self::place_block(generate_code(&su, mu), x as i32, y as i32, z as i32)
      })
      .register_fn("look_at", {
        let su = su.clone();
        move |mu: &str, x: f64, y: f64, z: f64| Self::look_at(generate_code(&su, mu), x, y, z)
      })
      .register_fn("look_at_block", {
        let su = su.clone();
        move |mu: &str, x: i64, y: i64, z: i64| {
          Self::look_at_block(generate_code(&su, mu), x as i32, y as i32, z as i32)
        }
      });

    engine
      .register_fn("drop_selected_item", {
        let su = su.clone();
        move |mu: &str, lock: bool| Self::drop_selected_item(generate_code(&su, mu), lock)
      })
      .register_fn("drop_all_items", {
        let su = su.clone();
        move |mu: &str, lock: bool| Self::drop_all_items(generate_code(&su, mu), lock)
      })
      .register_fn("inv_click", {
        let su = su.clone();
        move |mu: &str, slot: i64, mode: &str, lock: bool| {
          Self::inv_click(generate_code(&su, mu), slot as usize, mode, lock)
        }
      })
      .register_fn("inv_click_on", {
        let su = su.clone();
        move |mu: &str, name: &str, mode: &str, lock: bool| Self::inv_click_on(generate_code(&su, mu), name, mode, lock)
      })
      .register_fn("set_selected_slot", {
        let su = su.clone();
        move |mu: &str, slot: i64| Self::set_selected_slot(generate_code(&su, mu), slot as u8)
      });

    engine
      .register_fn("attack", {
        let su = su.clone();
        move |mu: &str, target: &str, distance: f64, look_at_target: bool| {
          Self::attack(generate_code(&su, mu), target.to_string(), distance, look_at_target);
        }
      })
      .register_fn("start_use_item", {
        let su = su.clone();
        move |mu: &str, hand: &str| Self::start_use_item(generate_code(&su, mu), hand)
      })
      .register_fn("release_use_item", {
        let su = su.clone();
        move |mu: &str| Self::release_use_item(generate_code(&su, mu))
      });

    engine
      .register_fn("set_state", {
        let su = su.clone();
        move |mu: &str, state_str: &str, value: bool| {
          if mu == "*" {
            return;
          }

          if let Some(state) = State::from_str(state_str) {
            try_setst(&(generate_code(&su, mu) as u8), state, value);
          }
        }
      })
      .register_fn("get_state", {
        let su = su.clone();
        move |mu: &str, state_str: &str| {
          if mu == "*" {
            return false;
          }

          if let Some(state) = State::from_str(state_str) {
            try_getst(&(generate_code(&su, mu) as u8), state)
          } else {
            false
          }
        }
      });

    match engine.eval::<()>(&script) {
      Ok(_) => {
        if !su.is_empty() {
          emit_log(format!("Сценарий автоматически выполнен для {}", su), "info");
        } else {
          emit_log("Сценарий выполнен", "info");
          emit_msg("Сценарий", "Сценарий выполнен");
        }
      }
      Err(err) => match err.as_ref() {
        &EvalAltResult::ErrorTerminated(_, _) => {
          emit_log("Сценарий принудительно остановлен", "info");
          emit_msg("Сценарий", "Сценарий принудительно остановлен");
        }
        _ => {
          if !su.is_empty() {
            emit_log(format!("Ошибка выполнения сценария для {}: {}", su, err), "info");
          } else {
            emit_log(format!("Ошибка выполнения сценария: {}", err), "error");
            emit_msg("Сценарий", "Не удалось выполнить сценарий");
          }
        }
      },
    }

    if su.is_empty() {
      EXECUTOR_GLOBAL_STATE.store(false, Ordering::SeqCst);
    } else {
      if let Ok(mut states) = EXECUTOR_STATE_SYSTEM.try_write() {
        states.remove(&su);
      }
    }
  }

  /// Метод остановки процесса исполнения пользовательского сценария
  pub async fn stop(&self) {
    EXECUTOR_GLOBAL_STATE.store(false, Ordering::SeqCst);
    EXECUTOR_STATE_SYSTEM.write().await.clear();
  }
}
