use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};

use azalea::core::position::BlockPos;
use azalea::prelude::PathfinderClientExt;
use azalea::{Client, Vec3, WalkDirection};
use salarixi_extensions::index::IndexExt;

use crate::bot::common::{get_average_coordinates_of_bots, this_is_solid_block};
use crate::bot::common_actions::{disconnect_bot, reset_bot};
use crate::bot::extensions::{
  go_to, BotDefaultExt, BotInventoryExt, BotMovementExt, BotPhysicsExt, BotRotationExt, ClickMode,
};
use crate::bot::quick::enumeration::QuickTaskName;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::bot::systems::registry::REGISTRY_SYSTEM;
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::launch::process::current_options;
use crate::server::transfer::{emit_log, emit_msg, LogClass};
use crate::tools::*;
use crate::webhook::send_webhook;
use crate::{sleep, take_bot};

static TASK_ACTIVITY: AtomicBool = AtomicBool::new(false);

pub async fn run_quick_task(id: u8) {
  if TASK_ACTIVITY.load(Ordering::SeqCst) {
    return;
  }

  TASK_ACTIVITY.store(true, Ordering::SeqCst);

  let Some(task_name) = QuickTaskName::from_index(id) else {
    return;
  };

  let connected_bots = PROFILE_SYSTEM.get_connected_count().await;
  let str_task_name = task_name.to_str();

  if let Some(opts) = current_options().await {
    if opts.basic.use_webhook && opts.webhook.send_actions {
      send_webhook(
        opts.webhook.url,
        format!("{} ботов получили быструю задачу \"{}\"", connected_bots, str_task_name),
      );
    }
  }

  emit_msg("Быстрая задача", format!("Быстрая задача \"{}\"", str_task_name));

  let mut tasks = Vec::new();

  for index in PROFILE_SYSTEM.get_all_connected().await.into_keys() {
    let task = tokio::spawn(async move {
      let mut success = false;

      take_bot!(&index, async |bot| {
        success = run_for_bot(bot, task_name, index).await;
      });

      success
    });

    tasks.push(task);
  }

  let mut success_count = 0;
  for task in tasks {
    if let Ok(true) = task.await {
      success_count += 1;
    }
  }

  TASK_ACTIVITY.store(false, Ordering::SeqCst);

  emit_log(
    format!(
      "Быструю задачу \"{}\" выполнили {} из {} ботов",
      str_task_name, success_count, connected_bots
    ),
    LogClass::Info,
  );
}

async fn run_for_bot(bot: &Client, task_name: QuickTaskName, index: u8) -> bool {
  match task_name {
    QuickTaskName::ClearInventory => {
      if let Some(menu) = bot.get_inventory_menu() {
        for (slot, _) in menu.slots().iter().enumerate() {
          bot.inventory_click(&index, slot, ClickMode::DropAll, false).await;
        }
      } else {
        return false;
      }
    }
    QuickTaskName::MoveForward => {
      bot.start_walking(&index, WalkDirection::Forward).await;
      sleep!(300);
      bot.stop_move(&index).await;
    }
    QuickTaskName::MoveBackward => {
      bot.start_walking(&index, WalkDirection::Backward).await;
      sleep!(300);
      bot.stop_move(&index).await;
    }
    QuickTaskName::MoveLeft => {
      bot.start_walking(&index, WalkDirection::Right).await;
      sleep!(300);
      bot.stop_move(&index).await;
    }
    QuickTaskName::MoveRight => {
      bot.start_walking(&index, WalkDirection::Left).await;
      sleep!(300);
      bot.stop_move(&index).await;
    }
    QuickTaskName::Jump => {
      bot.jump();
    }
    QuickTaskName::Shift => {
      bot.set_crouching(true);
      sleep!(200);
      bot.set_crouching(false);
    }
    QuickTaskName::Fly => {
      for i in 0..randnum(3, 5) {
        bot.set_velocity("y", randnum(0.022 * i as f64, 0.031 * i as f64));
        sleep!(50);
      }
    }
    QuickTaskName::Quit => {
      disconnect_bot(&index, true).await;
    }
    QuickTaskName::Rise => {
      if !getst(&index, State::CanLooking).await || !getst(&index, State::CanInteracting).await {
        return false;
      }

      let mut block_slot = None;

      if let Some(menu) = bot.get_inventory_menu() {
        for (slot, item) in menu.slots().iter().enumerate() {
          if !item.is_empty() {
            if this_is_solid_block(item.kind()) {
              block_slot = Some(slot);
              break;
            }
          }
        }
      }

      let Some(slot) = block_slot else {
        return false;
      };

      setmst(&index, StateName::Looking, true).await;
      setmst(&index, StateName::Interacting, true).await;

      bot.take_item(&index, slot, false).await;

      let initial_dir = bot.direction();

      bot.set_direction(
        initial_dir.y_rot() + randnum(-5.0, 5.0) as f32,
        randnum(40.0, 58.0) as f32,
      );

      sleep!(randnum(50, 100));

      bot.jump();

      bot.set_direction(
        bot.direction().y_rot() + randnum(-5.0, 5.0) as f32,
        randnum(86.0, 90.0) as f32,
      );

      sleep!(randnum(250, 300));

      let Some(foot_pos) = bot.foot_pos() else {
        return false;
      };

      let block_under = BlockPos::from(Vec3::new(foot_pos.x, foot_pos.y - 0.5, foot_pos.z));

      bot.block_interact(block_under);

      sleep!(randnum(100, 150));

      bot.set_direction(initial_dir.y_rot(), initial_dir.x_rot());

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;
    }
    QuickTaskName::Capsule => {
      let Some(foot_pos) = bot.foot_pos() else {
        return false;
      };

      let block_positions = vec![
        BlockPos {
          x: (foot_pos.x - 1.0).floor() as i32,
          y: foot_pos.y.floor() as i32,
          z: foot_pos.z.floor() as i32,
        },
        BlockPos {
          x: (foot_pos.x + 1.0).floor() as i32,
          y: foot_pos.y.floor() as i32,
          z: foot_pos.z.floor() as i32,
        },
        BlockPos {
          x: foot_pos.x.floor() as i32,
          y: foot_pos.y.floor() as i32,
          z: (foot_pos.z - 1.0).floor() as i32,
        },
        BlockPos {
          x: foot_pos.x.floor() as i32,
          y: foot_pos.y.floor() as i32,
          z: (foot_pos.z + 1.0).floor() as i32,
        },
        BlockPos {
          x: (foot_pos.x - 1.0).floor() as i32,
          y: (foot_pos.y + 1.0).floor() as i32,
          z: foot_pos.z.floor() as i32,
        },
        BlockPos {
          x: (foot_pos.x + 1.0).floor() as i32,
          y: (foot_pos.y + 1.0).floor() as i32,
          z: foot_pos.z.floor() as i32,
        },
        BlockPos {
          x: foot_pos.x.floor() as i32,
          y: (foot_pos.y + 1.0).floor() as i32,
          z: (foot_pos.z - 1.0).floor() as i32,
        },
        BlockPos {
          x: foot_pos.x.floor() as i32,
          y: (foot_pos.y + 1.0).floor() as i32,
          z: (foot_pos.z + 1.0).floor() as i32,
        },
        BlockPos {
          x: (foot_pos.x - 1.0).floor() as i32,
          y: (foot_pos.y + 2.0).floor() as i32,
          z: foot_pos.z.floor() as i32,
        },
        BlockPos {
          x: foot_pos.x.floor() as i32,
          y: (foot_pos.y + 2.0).floor() as i32,
          z: foot_pos.z.floor() as i32,
        },
      ];

      let mut count = 0;

      for pos in block_positions {
        let mut block_slot = None;

        if let Some(menu) = bot.get_inventory_menu() {
          for (slot, item) in menu.slots().iter().enumerate() {
            if !item.is_empty() {
              if this_is_solid_block(item.kind()) {
                block_slot = Some(slot);
                break;
              }
            }
          }
        }

        let Some(slot) = block_slot else {
          return false;
        };

        if !getst(&index, State::CanLooking).await || !getst(&index, State::CanInteracting).await {
          return false;
        };

        setmst(&index, StateName::Looking, true).await;
        setmst(&index, StateName::Interacting, true).await;

        bot.take_item(&index, slot, false).await;

        count = count + 1;

        if count == 10 {
          sleep!(randnum(150, 200));
        }

        if count == 9 {
          bot.jump();
          sleep!(50);
          bot.set_crouching(true);
          sleep!(randnum(100, 150));
        }

        bot.look_at_block(pos, false).await;
        sleep!(randnum(50, 100));
        bot.block_interact(pos);

        if count == 10 {
          bot.set_crouching(false);
        }

        setmst(&index, StateName::Looking, false).await;
        setmst(&index, StateName::Interacting, false).await;

        sleep!(randnum(100, 150));
      }
    }
    QuickTaskName::Unite => {
      let mut positions = vec![];

      for username in PROFILE_SYSTEM.get_all_connected().await.keys() {
        let Some(b) = REGISTRY_SYSTEM.get_bot(username) else {
          continue;
        };

        let Some(foot_pos) = b.foot_pos() else {
          continue;
        };

        positions.push(foot_pos);
      }

      let average_cords = get_average_coordinates_of_bots(&positions);

      go_to(index, average_cords.0 as i32, average_cords.2 as i32);
    }
    QuickTaskName::Turn => {
      let direction = bot.direction();
      bot.set_direction(direction.y_rot() - 90.0, direction.x_rot());
    }
    QuickTaskName::Zero => {
      bot.set_direction(0.0, 0.0);
    }
    QuickTaskName::FormCircle => {
      let mut positions = vec![];

      for username in PROFILE_SYSTEM.get_all_connected().await.keys() {
        let Some(b) = REGISTRY_SYSTEM.get_bot(username) else {
          continue;
        };

        let Some(foot_pos) = b.foot_pos() else {
          continue;
        };

        positions.push(foot_pos);
      }

      let average_cords = get_average_coordinates_of_bots(&positions);
      let angle = 2.0 * PI * (index as f32) / (positions.len() as f32);
      let x = average_cords.0 + positions.len() as f64 * 0.5 * angle.cos() as f64;
      let z = average_cords.2 + positions.len() as f64 * 0.5 * angle.sin() as f64;

      go_to(index, x as i32, z as i32);
    }
    QuickTaskName::FormLine => {
      let mut positions = vec![];

      for username in PROFILE_SYSTEM.get_all_connected().await.keys() {
        let Some(b) = REGISTRY_SYSTEM.get_bot(username) else {
          continue;
        };

        let Some(foot_pos) = b.foot_pos() else {
          continue;
        };

        positions.push(foot_pos);
      }

      let average_cords = get_average_coordinates_of_bots(&positions);
      let x = average_cords.0 + 1.0 * (index as f64 * 1.0);
      let z = average_cords.2 * (index as f64 * 1.0);

      go_to(index, x as i32, z as i32);
    }
    QuickTaskName::StopPathfinder => {
      bot.stop_pathfinding();
    }
    QuickTaskName::Reset => {
      reset_bot(&index, true).await;
    }
  }

  true
}
