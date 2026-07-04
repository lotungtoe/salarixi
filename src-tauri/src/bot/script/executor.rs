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
use crate::bot::registry::BOT_REGISTRY;
use crate::bot::systems::{try_getst, try_setst, State, PROFILE_SYSTEM};
use crate::launch::process::{current_options, process_is_active};
use crate::other::webhook::*;

pub static SCRIPT_EXECUTOR: Lazy<ScriptExecutor> = Lazy::new(|| ScriptExecutor::new());

static EXECUTOR_STATE_SYSTEM: Lazy<RwLock<HashMap<String, ()>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static EXECUTOR_GLOBAL_STATE: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Функция выбора юзернейма
fn select_username<'a>(su: &'a str, mu: &'a str) -> &'a str {
  if su.is_empty() {
    mu
  } else {
    su
  }
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
fn create_dynamic_profile(username: &str) -> Dynamic {
  let mut map = Map::new();

  if let Some(profile) = PROFILE_SYSTEM.try_get(username) {
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

  fn chat(username: &str, message: String) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.chat(&message);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.chat(&message);
      }
    }
  }

  fn jump(username: &str) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.jump();
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.jump();
      }
    }
  }

  fn swing_arm(username: &str) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.swing_arm();
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.swing_arm();
      }
    }
  }

  fn set_jumping(username: &str, state: bool) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.set_jumping(state);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.set_jumping(state);
      }
    }
  }

  fn set_crouching(username: &str, state: bool) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.set_crouching(state);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.set_crouching(state);
      }
    }
  }

  fn set_velocity(username: &str, axis: &str, velocity: f64) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.set_velocity(axis, velocity);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.set_velocity(axis, velocity);
      }
    }
  }

  fn set_on_ground(username: &str, on_ground: bool) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.set_on_ground(on_ground);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.set_on_ground(on_ground);
      }
    }
  }

  fn set_old_position(username: &str, x: f64, y: f64, z: f64) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.set_old_position(x, y, z);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.set_old_position(x, y, z);
      }
    }
  }

  fn set_no_jump_delay(username: &str, delay: u32) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.set_no_jump_delay(delay);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.set_no_jump_delay(delay);
      }
    }
  }

  fn set_was_touching_water(username: &str, state: bool) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.set_was_touching_water(state);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.set_was_touching_water(state);
      }
    }
  }

  fn set_has_impulse(username: &str, state: bool) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.set_has_impulse(state);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.set_has_impulse(state);
      }
    }
  }

  fn start_walking(username: &str, direction_name: &str) {
    let username_string = username.to_string();
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
      if username_string == "*" {
        for name in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = BOT_REGISTRY.get_bot(name) {
            bot.start_walking(direction).await;
          }
        }
      } else {
        if let Some(bot) = BOT_REGISTRY.get_bot(&username_string) {
          bot.start_walking(direction).await;
        }
      }
    });
  }

  fn start_sprinting(username: &str, direction_name: &str) {
    let username_string = username.to_string();
    let direction = match direction_name {
      "forward" => SprintDirection::Forward,
      "forward-left" => SprintDirection::ForwardLeft,
      "forward-right" => SprintDirection::ForwardRight,
      _ => return,
    };

    tokio::spawn(async move {
      if username_string == "*" {
        for name in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = BOT_REGISTRY.get_bot(name) {
            bot.start_sprinting(direction).await;
          }
        }
      } else {
        if let Some(bot) = BOT_REGISTRY.get_bot(&username_string) {
          bot.start_sprinting(direction).await;
        }
      }
    });
  }

  fn start_pathfinder(username: &str, x: i32, z: i32) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        go_to(name.to_string(), x, z);
      });
    } else {
      go_to(username.to_string(), x, z);
    }
  }

  fn stop_move(username: &str) {
    let username_string = username.to_string();

    tokio::spawn(async move {
      if username_string == "*" {
        for name in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = BOT_REGISTRY.get_bot(name) {
            bot.stop_move().await;
          }
        }
      } else {
        if let Some(bot) = BOT_REGISTRY.get_bot(&username_string) {
          bot.stop_move().await;
        }
      }
    });
  }

  fn stop_pathfinder(username: &str) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.stop_pathfinding();
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.stop_pathfinding();
      }
    }
  }

  fn look_at(username: &str, x: f64, y: f64, z: f64) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.look_at(Vec3 { x, y, z });
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.look_at(Vec3 { x, y, z });
      }
    }
  }

  fn look_at_block(username: &str, x: i32, y: i32, z: i32) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.look_at(BlockPos::new(x, y, z).center());
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.look_at(BlockPos::new(x, y, z).center());
      }
    }
  }

  fn place_block(username: &str, x: i32, y: i32, z: i32) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.block_interact(BlockPos::new(x, y, z));
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.block_interact(BlockPos::new(x, y, z));
      }
    }
  }

  fn drop_selected_item(username: &str, lock: bool) {
    let username_string = username.to_string();

    tokio::spawn(async move {
      if username_string == "*" {
        for name in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = BOT_REGISTRY.get_bot(name) {
            let selected_slot = convert_hotbar_slot_to_inventory_slot(bot.get_selected_slot());
            bot.inventory_click(selected_slot, ClickMode::DropAll, lock).await;
          }
        }
      } else {
        if let Some(bot) = BOT_REGISTRY.get_bot(&username_string) {
          let selected_slot = convert_hotbar_slot_to_inventory_slot(bot.get_selected_slot());
          bot.inventory_click(selected_slot, ClickMode::DropAll, lock).await;
        }
      }
    });
  }

  fn drop_all_items(username: &str, lock: bool) {
    let username_string = username.to_string();

    tokio::spawn(async move {
      if username_string == "*" {
        for name in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = BOT_REGISTRY.get_bot(name) {
            if let Some(menu) = bot.get_inventory_menu() {
              for (slot, _) in menu.slots().iter().enumerate() {
                bot.inventory_click(slot, ClickMode::DropAll, lock).await;
              }
            }
          }
        }
      } else {
        if let Some(bot) = BOT_REGISTRY.get_bot(&username_string) {
          if let Some(menu) = bot.get_inventory_menu() {
            for (slot, _) in menu.slots().iter().enumerate() {
              bot.inventory_click(slot, ClickMode::DropAll, lock).await;
            }
          }
        }
      }
    });
  }

  fn inv_click(username: &str, slot: usize, mode: &str, lock: bool) {
    let username_string = username.to_string();
    let mode_string = mode.to_string();

    tokio::spawn(async move {
      if username_string == "*" {
        for name in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = BOT_REGISTRY.get_bot(name) {
            bot
              .inventory_click(slot, ClickMode::from(mode_string.as_str()), lock)
              .await;
          }
        }
      } else {
        if let Some(bot) = BOT_REGISTRY.get_bot(&username_string) {
          bot
            .inventory_click(slot, ClickMode::from(mode_string.as_str()), lock)
            .await;
        }
      }
    });
  }

  fn inv_click_on(username: &str, name: &str, mode: &str, lock: bool) {
    let username_string = username.to_string();
    let name_string = name.to_string();
    let mode_string = mode.to_string();

    tokio::spawn(async move {
      if username_string == "*" {
        for name in PROFILE_SYSTEM.get_all_connected().await.keys() {
          if let Some(bot) = BOT_REGISTRY.get_bot(name) {
            bot
              .inventory_click_on(&name_string, ClickMode::from(mode_string.as_str()), lock)
              .await;
          }
        }
      } else {
        if let Some(bot) = BOT_REGISTRY.get_bot(&username_string) {
          bot
            .inventory_click_on(&name_string, ClickMode::from(mode_string.as_str()), lock)
            .await;
        }
      }
    });
  }

  fn set_selected_slot(username: &str, slot: u8) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.set_selected_hotbar_slot(slot);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.set_selected_hotbar_slot(slot);
      }
    }
  }

  fn attack(username: &str, target: String, distance: f64, look_at_target: bool) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          if let Some(entity) = bot.find_nearest_entity(EntityType::from(target.clone()), distance) {
            if look_at_target {
              bot.look_at_entity(entity, false);
            }

            bot.attack(entity);
          }
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        if let Some(entity) = bot.find_nearest_entity(EntityType::from(target), distance) {
          if look_at_target {
            bot.look_at_entity(entity, false);
          }

          bot.attack(entity);
        }
      }
    }
  }

  fn start_use_item(username: &str, hand: &str) {
    let interaction_hand = if hand == "offhand" {
      InteractionHand::OffHand
    } else {
      InteractionHand::MainHand
    };

    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        BOT_REGISTRY.get_bot(name).map(|bot| {
          bot.start_use_held_item(interaction_hand);
        });
      });
    } else {
      BOT_REGISTRY.get_bot(username).map(|bot| {
        bot.start_use_held_item(interaction_hand);
      });
    }
  }

  fn release_use_item(username: &str) {
    if username == "*" {
      PROFILE_SYSTEM.try_get_all_connected().keys().for_each(|name| {
        BOT_REGISTRY.get_bot(name).map(|bot| {
          bot.release_use_held_item();
        });
      });
    } else {
      BOT_REGISTRY.get_bot(username).map(|bot| {
        bot.release_use_held_item();
      });
    }
  }

  /// Метод исполнения пользовательского сценария
  pub fn execute(&self, su: String, script: String) {
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
          BOT_REGISTRY
            .get_bot(select_username(&su, mu))
            .map(|b| b.id().0 as i64)
            .unwrap_or(0)
        }
      })
      .register_fn("get_bot_ping", {
        let su = su.clone();
        move |mu: &str| {
          BOT_REGISTRY
            .get_bot(select_username(&su, mu))
            .map(|b| b.ping() as i64)
            .unwrap_or(0)
        }
      })
      .register_fn("get_bot_feet_pos", {
        let su = su.clone();
        move |mu: &str| {
          BOT_REGISTRY
            .get_bot(select_username(&su, mu))
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
          BOT_REGISTRY
            .get_bot(select_username(&su, mu))
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
          BOT_REGISTRY
            .get_bot(select_username(&su, mu))
            .map(|b| b.get_health() as f64)
            .unwrap_or(0.0)
        }
      })
      .register_fn("get_bot_satiety", {
        let su = su.clone();
        move |mu: &str| {
          BOT_REGISTRY
            .get_bot(select_username(&su, mu))
            .map(|b| b.get_satiety() as i64)
            .unwrap_or(0)
        }
      })
      .register_fn("bot_is_workable", {
        let su = su.clone();
        move |mu: &str| {
          BOT_REGISTRY
            .get_bot(select_username(&su, mu))
            .map(|b| b.workable())
            .unwrap_or(false)
        }
      })
      .register_fn("get_bot_profile", {
        let su = su.clone();
        move |mu: &str| create_dynamic_profile(select_username(&su, mu))
      });

    if !su.is_empty() {
      engine
        .register_fn("current_bot_id", {
          let su = su.clone();
          move || BOT_REGISTRY.get_bot(&su).map(|b| b.id().0 as i64).unwrap_or(0)
        })
        .register_fn("current_bot_ping", {
          let su = su.clone();
          move || BOT_REGISTRY.get_bot(&su).map(|b| b.ping() as i64).unwrap_or(0)
        })
        .register_fn("current_bot_feet_pos", {
          let su = su.clone();
          move || {
            BOT_REGISTRY
              .get_bot(&su)
              .map(|b| {
                let pos = b.feet_pos();
                create_dynamic_vec(pos.x, pos.y, pos.z)
              })
              .unwrap_or(create_dynamic_vec(0.0, 0.0, 0.0))
          }
        })
        .register_fn("current_bot_eye_pos", {
          let su = su.clone();
          move || {
            BOT_REGISTRY
              .get_bot(&su)
              .map(|b| {
                let pos = b.eye_pos();
                create_dynamic_vec(pos.x, pos.y, pos.z)
              })
              .unwrap_or(create_dynamic_vec(0.0, 0.0, 0.0))
          }
        })
        .register_fn("current_bot_health", {
          let su = su.clone();
          move || BOT_REGISTRY.get_bot(&su).map(|b| b.get_health() as f64).unwrap_or(0.0)
        })
        .register_fn("current_bot_satiety", {
          let su = su.clone();
          move || BOT_REGISTRY.get_bot(&su).map(|b| b.get_satiety() as i64).unwrap_or(0)
        })
        .register_fn("current_bot_profile", {
          let su = su.clone();
          move || create_dynamic_profile(&su)
        });
    }

    engine
      .register_fn("chat", {
        let su = su.clone();
        move |mu: &str, message: &str| Self::chat(select_username(&su, mu), message.to_string())
      })
      .register_fn("jump", {
        let su = su.clone();
        move |mu: &str| Self::jump(select_username(&su, mu))
      })
      .register_fn("swing_arm", {
        let su = su.clone();
        move |mu: &str| Self::swing_arm(select_username(&su, mu))
      })
      .register_fn("set_jumping", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_jumping(select_username(&su, mu), state)
      })
      .register_fn("set_crouching", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_crouching(select_username(&su, mu), state)
      })
      .register_fn("set_velocity", {
        let su = su.clone();
        move |mu: &str, axis: &str, velocity: f64| Self::set_velocity(select_username(&su, mu), axis, velocity)
      })
      .register_fn("set_on_ground", {
        let su = su.clone();
        move |mu: &str, on_ground: bool| Self::set_on_ground(select_username(&su, mu), on_ground)
      })
      .register_fn("set_old_position", {
        let su = su.clone();
        move |mu: &str, x: f64, y: f64, z: f64| Self::set_old_position(select_username(&su, mu), x, y, z)
      })
      .register_fn("set_no_jump_delay", {
        let su = su.clone();
        move |mu: &str, delay: i64| Self::set_no_jump_delay(select_username(&su, mu), delay as u32)
      })
      .register_fn("set_was_touching_water", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_was_touching_water(select_username(&su, mu), state)
      })
      .register_fn("set_has_impulse", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_has_impulse(select_username(&su, mu), state)
      })
      .register_fn("start_walking", {
        let su = su.clone();
        move |mu: &str, direction: &str| Self::start_walking(select_username(&su, mu), direction)
      })
      .register_fn("start_sprinting", {
        let su = su.clone();
        move |mu: &str, direction: &str| Self::start_sprinting(select_username(&su, mu), direction)
      })
      .register_fn("start_pathfinder", {
        let su = su.clone();
        move |mu: &str, x: i64, z: i64| Self::start_pathfinder(select_username(&su, mu), x as i32, z as i32)
      })
      .register_fn("stop_move", {
        let su = su.clone();
        move |mu: &str| Self::stop_move(select_username(&su, mu))
      })
      .register_fn("stop_pathfinder", {
        let su = su.clone();
        move |mu: &str| Self::stop_pathfinder(select_username(&su, mu))
      });

    engine
      .register_fn("place_block", {
        let su = su.clone();
        move |mu: &str, x: i64, y: i64, z: i64| {
          Self::place_block(select_username(&su, mu), x as i32, y as i32, z as i32)
        }
      })
      .register_fn("look_at", {
        let su = su.clone();
        move |mu: &str, x: f64, y: f64, z: f64| Self::look_at(select_username(&su, mu), x, y, z)
      })
      .register_fn("look_at_block", {
        let su = su.clone();
        move |mu: &str, x: i64, y: i64, z: i64| {
          Self::look_at_block(select_username(&su, mu), x as i32, y as i32, z as i32)
        }
      });

    engine
      .register_fn("drop_selected_item", {
        let su = su.clone();
        move |mu: &str, lock: bool| Self::drop_selected_item(select_username(&su, mu), lock)
      })
      .register_fn("drop_all_items", {
        let su = su.clone();
        move |mu: &str, lock: bool| Self::drop_all_items(select_username(&su, mu), lock)
      })
      .register_fn("inv_click", {
        let su = su.clone();
        move |mu: &str, slot: i64, mode: &str, lock: bool| {
          Self::inv_click(select_username(&su, mu), slot as usize, mode, lock)
        }
      })
      .register_fn("inv_click_on", {
        let su = su.clone();
        move |mu: &str, name: &str, mode: &str, lock: bool| {
          Self::inv_click_on(select_username(&su, mu), name, mode, lock)
        }
      })
      .register_fn("set_selected_slot", {
        let su = su.clone();
        move |mu: &str, slot: i64| Self::set_selected_slot(select_username(&su, mu), slot as u8)
      });

    engine
      .register_fn("attack", {
        let su = su.clone();
        move |mu: &str, target: &str, distance: f64, look_at_target: bool| {
          Self::attack(select_username(&su, mu), target.to_string(), distance, look_at_target);
        }
      })
      .register_fn("start_use_item", {
        let su = su.clone();
        move |mu: &str, hand: &str| Self::start_use_item(select_username(&su, mu), hand)
      })
      .register_fn("release_use_item", {
        let su = su.clone();
        move |mu: &str| Self::release_use_item(select_username(&su, mu))
      });

    engine
      .register_fn("set_state", {
        let su = su.clone();
        move |mu: &str, state_str: &str, value: bool| {
          if let Some(state) = State::from_str(state_str) {
            try_setst(select_username(&su, mu), state, value);
          }
        }
      })
      .register_fn("get_state", {
        let su = su.clone();
        move |mu: &str, state_str: &str| {
          if let Some(state) = State::from_str(state_str) {
            try_getst(select_username(&su, mu), state)
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
