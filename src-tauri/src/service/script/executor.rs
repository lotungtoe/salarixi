use azalea::bot::BotClientExt;
use azalea::prelude::PathfinderClientExt;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::{BlockPos, SprintDirection, Vec3, WalkDirection};
use once_cell::sync::Lazy;
use rhai::{Dynamic, Engine, EvalAltResult, Map};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::api::emit::{emit_log, emit_msg};
use crate::service::core::bot::{get_state, set_state, State, PROFILES};
use crate::service::core::common::convert_hotbar_slot_to_inventory_slot;
use crate::service::core::extensions::{
  entity_type_from, go_to, BotDefaultExt, BotInteractExt, BotInventoryExt, BotMovementExt, BotPhysicsExt,
  BotRotationExt, InvClick,
};
use crate::service::core::registry::BOT_REGISTRY;
use crate::service::core::{active_bots_count, current_options};
use crate::service::webhook::*;

pub static SCRIPT_EXECUTOR: Lazy<ScriptExecutor> = Lazy::new(|| ScriptExecutor::new());

static EXECUTOR_STATE: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Функция парсинга юзернейма
fn parse_username<'a>(su: &'a str, mu: &'a str) -> &'a str {
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

  if let Some(profile) = PROFILES.get(username) {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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

  fn start_walking(username: &str, direction_name: String) {
    let direction = match direction_name.as_str() {
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

    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.start_walking(direction);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.start_walking(direction);
      }
    }
  }

  fn start_sprinting(username: &str, direction_name: &str) {
    let direction = match direction_name {
      "forward" => SprintDirection::Forward,
      "forward-left" => SprintDirection::ForwardLeft,
      "forward-right" => SprintDirection::ForwardRight,
      _ => return,
    };

    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.start_sprinting(direction);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.start_sprinting(direction);
      }
    }
  }

  fn start_pathfinder(username: &str, x: i32, z: i32) {
    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
        go_to(name.to_string(), x, z);
      });
    } else {
      go_to(username.to_string(), x, z);
    }
  }

  fn stop_move(username: &str) {
    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.stop_move();
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.stop_move();
      }
    }
  }

  fn stop_pathfinder(username: &str) {
    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          let selected_slot = convert_hotbar_slot_to_inventory_slot(bot.get_selected_slot());
          bot.inventory_click(selected_slot, InvClick::from(4), lock);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        let selected_slot = convert_hotbar_slot_to_inventory_slot(bot.get_selected_slot());
        bot.inventory_click(selected_slot, InvClick::from(4), lock);
      }
    }
  }

  fn drop_all_items(username: &str, lock: bool) {
    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          if let Some(menu) = bot.get_inventory_menu() {
            for (slot, _) in menu.slots().iter().enumerate() {
              bot.inventory_click(slot, InvClick::from(4), lock);
            }
          }
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        if let Some(menu) = bot.get_inventory_menu() {
          for (slot, _) in menu.slots().iter().enumerate() {
            bot.inventory_click(slot, InvClick::from(4), lock);
          }
        }
      }
    }
  }

  fn inv_click(username: &str, slot: usize, click_mode: u8, lock: bool) {
    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.inventory_click(slot, InvClick::from(click_mode), lock);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.inventory_click(slot, InvClick::from(click_mode), lock);
      }
    }
  }

  fn inv_click_on(username: &str, name: &str, click_mode: u8, lock: bool) {
    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          bot.inventory_click_on(name, InvClick::from(click_mode), lock);
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        bot.inventory_click_on(name, InvClick::from(click_mode), lock);
      }
    }
  }

  fn set_selected_slot(username: &str, slot: u8) {
    if username == "*" {
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
        if let Some(bot) = BOT_REGISTRY.get_bot(name) {
          if let Some(entity) = bot.find_nearest_entity(entity_type_from(target.clone()), distance) {
            if look_at_target {
              bot.look_at_entity(entity, false);
            }

            bot.attack(entity);
          }
        }
      });
    } else {
      if let Some(bot) = BOT_REGISTRY.get_bot(username) {
        if let Some(entity) = bot.find_nearest_entity(entity_type_from(target), distance) {
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
      PROFILES.get_all().keys().for_each(|name| {
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
      PROFILES.get_all().keys().for_each(|name| {
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
    EXECUTOR_STATE.store(true, Ordering::Relaxed);

    let mut engine = Engine::new();

    engine.on_progress(move |_| {
      if !EXECUTOR_STATE.load(Ordering::Relaxed) {
        Some(Dynamic::UNIT)
      } else {
        None
      }
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
        if let Some(opts) = current_options() {
          send_webhook(opts.webhook.url, content.to_string());
        }
      });

    engine.register_fn("active_bots_count", || active_bots_count());

    engine
      .register_fn("get_bot_id", {
        let su = su.clone();
        move |mu: &str| {
          BOT_REGISTRY
            .get_bot(parse_username(&su, mu))
            .map(|b| b.id().0 as i64)
            .unwrap_or(0)
        }
      })
      .register_fn("get_bot_ping", {
        let su = su.clone();
        move |mu: &str| {
          BOT_REGISTRY
            .get_bot(parse_username(&su, mu))
            .map(|b| b.ping() as i64)
            .unwrap_or(0)
        }
      })
      .register_fn("get_bot_feet_pos", {
        let su = su.clone();
        move |mu: &str| {
          BOT_REGISTRY
            .get_bot(parse_username(&su, mu))
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
            .get_bot(parse_username(&su, mu))
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
            .get_bot(parse_username(&su, mu))
            .map(|b| b.get_health() as f64)
            .unwrap_or(0.0)
        }
      })
      .register_fn("get_bot_satiety", {
        let su = su.clone();
        move |mu: &str| {
          BOT_REGISTRY
            .get_bot(parse_username(&su, mu))
            .map(|b| b.get_satiety() as i64)
            .unwrap_or(0)
        }
      })
      .register_fn("bot_is_workable", {
        let su = su.clone();
        move |mu: &str| {
          BOT_REGISTRY
            .get_bot(parse_username(&su, mu))
            .map(|b| b.workable())
            .unwrap_or(false)
        }
      })
      .register_fn("get_bot_profile", {
        let su = su.clone();
        move |mu: &str| create_dynamic_profile(parse_username(&su, mu))
      });

    engine
      .register_fn("chat", {
        let su = su.clone();
        move |mu: &str, message: &str| Self::chat(parse_username(&su, mu), message.to_string())
      })
      .register_fn("jump", {
        let su = su.clone();
        move |mu: &str| Self::jump(parse_username(&su, mu))
      })
      .register_fn("swing_arm", {
        let su = su.clone();
        move |mu: &str| Self::swing_arm(parse_username(&su, mu))
      })
      .register_fn("set_jumping", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_jumping(parse_username(&su, mu), state)
      })
      .register_fn("set_crouching", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_crouching(parse_username(&su, mu), state)
      })
      .register_fn("set_velocity", {
        let su = su.clone();
        move |mu: &str, axis: &str, velocity: f64| Self::set_velocity(parse_username(&su, mu), axis, velocity)
      })
      .register_fn("set_on_ground", {
        let su = su.clone();
        move |mu: &str, on_ground: bool| Self::set_on_ground(parse_username(&su, mu), on_ground)
      })
      .register_fn("set_old_position", {
        let su = su.clone();
        move |mu: &str, x: f64, y: f64, z: f64| Self::set_old_position(parse_username(&su, mu), x, y, z)
      })
      .register_fn("set_no_jump_delay", {
        let su = su.clone();
        move |mu: &str, delay: i64| Self::set_no_jump_delay(parse_username(&su, mu), delay as u32)
      })
      .register_fn("set_was_touching_water", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_was_touching_water(parse_username(&su, mu), state)
      })
      .register_fn("set_has_impulse", {
        let su = su.clone();
        move |mu: &str, state: bool| Self::set_has_impulse(parse_username(&su, mu), state)
      })
      .register_fn("start_walking", {
        let su = su.clone();
        move |mu: &str, direction: &str| Self::start_walking(parse_username(&su, mu), direction.to_string())
      })
      .register_fn("start_sprinting", {
        let su = su.clone();
        move |mu: &str, direction: &str| Self::start_sprinting(parse_username(&su, mu), direction)
      })
      .register_fn("start_pathfinder", {
        let su = su.clone();
        move |mu: &str, x: i64, z: i64| Self::start_pathfinder(parse_username(&su, mu), x as i32, z as i32)
      })
      .register_fn("stop_move", {
        let su = su.clone();
        move |mu: &str| Self::stop_move(parse_username(&su, mu))
      })
      .register_fn("stop_pathfinder", {
        let su = su.clone();
        move |mu: &str| Self::stop_pathfinder(parse_username(&su, mu))
      });

    engine
      .register_fn("place_block", {
        let su = su.clone();
        move |mu: &str, x: i64, y: i64, z: i64| Self::place_block(parse_username(&su, mu), x as i32, y as i32, z as i32)
      })
      .register_fn("look_at", {
        let su = su.clone();
        move |mu: &str, x: f64, y: f64, z: f64| Self::look_at(parse_username(&su, mu), x, y, z)
      })
      .register_fn("look_at_block", {
        let su = su.clone();
        move |mu: &str, x: i64, y: i64, z: i64| {
          Self::look_at_block(parse_username(&su, mu), x as i32, y as i32, z as i32)
        }
      });

    engine
      .register_fn("drop_selected_item", {
        let su = su.clone();
        move |mu: &str, lock: bool| Self::drop_selected_item(parse_username(&su, mu), lock)
      })
      .register_fn("drop_all_items", {
        let su = su.clone();
        move |mu: &str, lock: bool| Self::drop_all_items(parse_username(&su, mu), lock)
      })
      .register_fn("inv_click", {
        let su = su.clone();
        move |mu: &str, slot: i64, mode: i64, lock: bool| {
          Self::inv_click(parse_username(&su, mu), slot as usize, mode as u8, lock)
        }
      })
      .register_fn("inv_click_on", {
        let su = su.clone();
        move |mu: &str, name: &str, mode: i64, lock: bool| {
          Self::inv_click_on(parse_username(&su, mu), name, mode as u8, lock)
        }
      })
      .register_fn("set_selected_slot", {
        let su = su.clone();
        move |mu: &str, slot: i64| Self::set_selected_slot(parse_username(&su, mu), slot as u8)
      });

    engine
      .register_fn("attack", {
        let su = su.clone();
        move |mu: &str, target: &str, distance: f64, look_at_target: bool| {
          Self::attack(parse_username(&su, mu), target.to_string(), distance, look_at_target);
        }
      })
      .register_fn("start_use_item", {
        let su = su.clone();
        move |mu: &str, hand: &str| Self::start_use_item(parse_username(&su, mu), hand)
      })
      .register_fn("release_use_item", {
        let su = su.clone();
        move |mu: &str| Self::release_use_item(parse_username(&su, mu))
      });

    engine
      .register_fn("set_state", {
        let su = su.clone();
        move |mu: &str, state_str: &str, value: bool| {
          if let Some(state) = State::from_str(state_str) {
            set_state(parse_username(&su, mu), state, value);
          }
        }
      })
      .register_fn("get_state", {
        let su = su.clone();
        move |mu: &str, state_str: &str| {
          if let Some(state) = State::from_str(state_str) {
            get_state(parse_username(&su, mu), state);
          }
        }
      });

    match engine.eval::<()>(&script) {
      Ok(_) => {
        if !su.is_empty() {
          emit_log(format!("Скрипт автоматически выполнен для {}", su), "info");
        } else {
          emit_log("Скрипт выполнен", "info");
          emit_msg("Скриптинг", "Скрипт выполнен");
        }
      }
      Err(err) => match err.as_ref() {
        &EvalAltResult::ErrorTerminated(_, _) => {
          emit_log("Скрипт принудительно остановлен", "info");
          emit_msg("Скриптинг", "Скрипт принудительно остановлен");
        }
        _ => {
          if !su.is_empty() {
            emit_log(format!("Ошибка выполнения скрипта для {}: {}", su, err), "info");
          } else {
            emit_log(format!("Ошибка выполнения скрипта: {}", err), "error");
            emit_msg("Скриптинг", "Не удалось выполнить скрипт");
          }
        }
      },
    }

    EXECUTOR_STATE.store(false, Ordering::Relaxed);
  }

  /// Метод остановки процесса исполнения пользовательского сценария
  pub fn stop(&self) {
    if EXECUTOR_STATE.load(Ordering::Relaxed) {
      EXECUTOR_STATE.store(false, Ordering::Relaxed);
    }
  }
}
