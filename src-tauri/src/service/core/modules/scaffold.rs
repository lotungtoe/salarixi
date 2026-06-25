use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::{Vec3, WalkDirection};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::common::{
  convert_hotbar_slot_to_inventory_slot, convert_inventory_slot_to_hotbar_slot, get_block_state, this_is_solid_block,
};
use crate::service::core::controllers::{gettskact, killtsk, pushetsk, pushrtsk, setmst, StateName, TaskName};
use crate::service::core::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt, BotPhysicsExt};
use crate::service::core::traits::SalarixiModule;
use crate::service::tools::*;
use crate::take_bot;

pub struct ScaffoldModule;

#[derive(Deserialize)]
pub struct ScaffoldOptions {
  pub mode: String,
  pub delay: Option<u64>,
  pub min_gaze_degree_x: Option<f32>,
  pub max_gaze_degree_x: Option<f32>,
  pub state: bool,
}

impl ScaffoldModule {
  async fn take_block(bot: &Client) -> bool {
    if let Some(menu) = bot.get_inventory_menu() {
      if let Some(item) = menu.slot(convert_hotbar_slot_to_inventory_slot(bot.get_selected_slot())) {
        if this_is_solid_block(item.kind()) {
          return true;
        }
      }

      let mut block_slot = None;

      for (slot, item) in menu.slots().iter().enumerate() {
        if this_is_solid_block(item.kind()) {
          for s in 36..=44 {
            if let Some(i) = menu.slot(s) {
              if this_is_solid_block(i.kind()) {
                if let Some(hotbar_slot) = convert_inventory_slot_to_hotbar_slot(s) {
                  if bot.get_selected_slot() == hotbar_slot {
                    return true;
                  }
                }
              }
            }
          }

          if let Some(hotbar_slot) = convert_inventory_slot_to_hotbar_slot(slot) {
            if bot.get_selected_slot() == hotbar_slot {
              return true;
            }
          }

          block_slot = Some(slot);
          break;
        }
      }

      if let Some(slot) = block_slot {
        bot.take_item(slot, true).await;
        return true;
      }
    }

    false
  }

  fn simulate_inaccuracy(bot: &Client, direction: (f32, f32)) {
    let inaccurate_direction = (
      direction.0 + randnum(-0.08, 0.08) as f32,
      direction.1 + randnum(-0.08, 0.08) as f32,
    );

    bot.set_direction(inaccurate_direction.0, inaccurate_direction.1);
  }

  fn direct_gaze(bot: &Client, min_x_rot: Option<f32>, max_x_rot: Option<f32>) {
    let direction = bot.direction();

    let min_x = if let Some(rot) = min_x_rot { rot } else { 80.0 } as f64;
    let max_x = if let Some(rot) = max_x_rot { rot } else { 83.0 } as f64;

    bot.set_direction(direction.0, randnum(min_x, max_x) as f32);
  }

  async fn go_back(username: String) {
    let username_clone = username.clone();
    let extra_task = tokio::spawn(async move {
      loop {
        take_bot!(&username_clone, async |bot| {
          bot.start_walking(WalkDirection::Backward).await;
        });

        sleep(Duration::from_millis(100)).await;
      }
    });

    pushetsk(&username, TaskName::Scaffold, extra_task).await;
  }

  async fn noob_bridge_scaffold(bot: &Client, options: &ScaffoldOptions) {
    loop {
      if !bot.crouching() {
        bot.set_crouching(true);
      }

      if Self::take_block(bot).await {
        Self::direct_gaze(bot, options.min_gaze_degree_x, options.max_gaze_degree_x);

        let position = bot.position();
        let block_under = BlockPos::new(
          position.x.floor() as i32,
          (position.y - 0.5).floor() as i32,
          position.z.floor() as i32,
        );

        let is_air = if let Some(state) = get_block_state(bot, block_under) {
          state.is_air()
        } else {
          false
        };

        if is_air {
          bot.swing_arm();

          bot.start_use_item();

          sleep(Duration::from_millis(randnum(100, 200))).await;

          Self::simulate_inaccuracy(bot, bot.direction());

          sleep(Duration::from_millis(randnum(100, 150))).await;
        }
      }

      sleep(Duration::from_millis(options.delay.unwrap_or(25))).await;
    }
  }

  async fn ninja_bridge_scaffold(bot: &Client, options: &ScaffoldOptions) {
    loop {
      if Self::take_block(bot).await {
        Self::direct_gaze(bot, options.min_gaze_degree_x, options.max_gaze_degree_x);

        let pos = bot.position();
        let block_under = BlockPos::new(pos.x.floor() as i32, (pos.y - 0.5).floor() as i32, pos.z.floor() as i32);

        let is_air = if let Some(state) = get_block_state(bot, block_under) {
          state.is_air()
        } else {
          false
        };

        if is_air {
          bot.set_crouching(true);

          sleep(Duration::from_millis(50)).await;

          bot.swing_arm();

          bot.start_use_item();

          sleep(Duration::from_millis(randnum(50, 100))).await;

          Self::simulate_inaccuracy(bot, bot.direction());

          sleep(Duration::from_millis(50)).await;

          bot.set_crouching(false);
        }
      }

      sleep(Duration::from_millis(options.delay.unwrap_or(25))).await;
    }
  }

  async fn god_bridge_scaffold(bot: &Client, options: &ScaffoldOptions) {
    loop {
      if Self::take_block(bot).await {
        Self::direct_gaze(bot, options.min_gaze_degree_x, options.max_gaze_degree_x);

        let position = bot.position();
        let block_under = BlockPos::new(
          position.x.floor() as i32,
          (position.y - 0.5).floor() as i32,
          position.z.floor() as i32,
        );

        let is_air = if let Some(state) = get_block_state(bot, block_under) {
          state.is_air()
        } else {
          false
        };

        if is_air {
          bot.swing_arm();

          bot.start_use_item();

          Self::simulate_inaccuracy(bot, bot.direction());
        }
      }

      sleep(Duration::from_millis(options.delay.unwrap_or(25))).await;
    }
  }

  async fn jump_bridge_scaffold(bot: &Client, options: &ScaffoldOptions) {
    loop {
      if Self::take_block(bot).await {
        Self::direct_gaze(bot, options.min_gaze_degree_x, options.max_gaze_degree_x);

        let position = bot.position();
        let velocity = if let Some(physics) = bot.get_physics() {
          physics.velocity
        } else {
          Vec3::ZERO
        };

        let block_under = BlockPos::new(
          position.x.floor() as i32,
          (if velocity.y != 0.0 {
            position.y - 1.0
          } else {
            position.y - 0.5
          })
          .floor() as i32,
          position.z.floor() as i32,
        );

        let is_air = if let Some(state) = get_block_state(bot, block_under) {
          state.is_air()
        } else {
          false
        };

        if is_air {
          bot.jump();

          bot.swing_arm();

          bot.start_use_item();

          Self::simulate_inaccuracy(bot, bot.direction());
        }
      }

      sleep(Duration::from_millis(options.delay.unwrap_or(25))).await;
    }
  }

  async fn start(bot: &Client, options: &ScaffoldOptions) {
    Self::go_back(bot.username()).await;

    match options.mode.as_str() {
      "noob-bridge" => Self::noob_bridge_scaffold(bot, &options).await,
      "ninja-bridge" => Self::ninja_bridge_scaffold(bot, &options).await,
      "god-bridge" => Self::god_bridge_scaffold(bot, &options).await,
      "jump-bridge" => Self::jump_bridge_scaffold(bot, &options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<ScaffoldOptions> for ScaffoldModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, username: String, options: ScaffoldOptions) -> bool {
    if options.state && !gettskact(&username, TaskName::Scaffold).await {
      let username_clone = username.clone();
      let task_handle = tokio::spawn(async move {
        take_bot!(&username_clone, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&username, TaskName::Scaffold, task_handle).await;
    } else if !options.state && gettskact(&username, TaskName::Scaffold).await {
      killtsk(&username, TaskName::Scaffold).await;

      take_bot!(&username, async |bot| {
        bot.set_crouching(false);
        bot.stop_move().await;
      });

      setmst(&username, StateName::Looking, false).await;
      setmst(&username, StateName::Interacting, false).await;
    } else {
      return false;
    }

    true
  }
}
