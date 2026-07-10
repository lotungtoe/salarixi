use std::f32::consts::PI;

use azalea::bot::BotClientExt;
use azalea::core::position::BlockPos;
use azalea::prelude::PathfinderClientExt;
use azalea::{Vec3, WalkDirection};
use once_cell::sync::Lazy;

use crate::bot::common::{get_average_coordinates_of_bots, this_is_solid_block};
use crate::bot::extensions::{go_to, BotDefaultExt, BotInventoryExt, BotMovementExt, BotPhysicsExt, ClickMode};
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::bot::systems::registry::REGISTRY_SYSTEM;
use crate::bot::systems::states::{getst, setmst, State, StateName, STATE_SYSTEM};
use crate::bot::systems::tasks::TASK_SYSTEM;
use crate::bot::PLUGINS;
use crate::other::tools::*;
use crate::{sleep, take_bot};

pub static QUICK_TASKS: Lazy<QuickTaskManager> = Lazy::new(|| QuickTaskManager::new());

/// Менеджер быстрых задач
pub struct QuickTaskManager;

impl QuickTaskManager {
  /// Метод создания менеджера
  pub fn new() -> Self {
    Self
  }

  /// Метод выполненя быстрой задачи
  pub async fn execute(&self, name: String) {
    for (number, index) in PROFILE_SYSTEM.get_all_connected().await.into_keys().enumerate() {
      let name = name.clone();

      tokio::spawn(async move {
        take_bot!(&index, async |bot| match name.as_str() {
          "clear-inventory" => {
            if let Some(menu) = bot.get_inventory_menu() {
              for (slot, _) in menu.slots().iter().enumerate() {
                bot.inventory_click(&index, slot, ClickMode::DropAll, true).await;
              }
            }
          }
          "move-forward" => {
            bot.start_walking(&index, WalkDirection::Forward).await;
            sleep!(200);
            bot.stop_move(&index).await;
          }
          "move-backward" => {
            bot.start_walking(&index, WalkDirection::Backward).await;
            sleep!(200);
            bot.stop_move(&index).await;
          }
          "move-left" => {
            bot.start_walking(&index, WalkDirection::Right).await;
            sleep!(200);
            bot.stop_move(&index).await;
          }
          "move-right" => {
            bot.start_walking(&index, WalkDirection::Left).await;
            sleep!(200);
            bot.stop_move(&index).await;
          }
          "jump" => {
            bot.jump();
          }
          "shift" => {
            bot.set_crouching(true);
            sleep!(200);
            bot.set_crouching(false);
          }
          "fly" => {
            for i in 0..randnum(3, 5) {
              bot.set_velocity("y", randnum(0.022 * i as f64, 0.031 * i as f64));
              sleep!(50);
            }
          }
          "quit" => {
            PLUGINS.kill_all_tasks_for(&index).await;
            TASK_SYSTEM.kill_all_tasks_for(&index).await;

            sleep!(2000);

            bot.disconnect();

            STATE_SYSTEM.remove(&index).await;
            TASK_SYSTEM.remove(&index).await;

            PROFILE_SYSTEM.set_str(&index, "status", "Оффлайн").await;
          }
          "rise" => {
            if getst(&index, State::CanLooking).await && getst(&index, State::CanInteracting).await {
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

              if let Some(slot) = block_slot {
                setmst(&index, StateName::Looking, true).await;
                setmst(&index, StateName::Interacting, true).await;

                bot.take_item(&index, slot, true).await;

                let original_direction_1 = bot.direction();

                bot.set_direction(
                  original_direction_1.0 + randnum(-5.0, 5.0) as f32,
                  randnum(40.0, 58.0) as f32,
                );

                sleep!(randnum(50, 100));

                bot.jump();

                let original_direction_2 = bot.direction();

                bot.set_direction(
                  original_direction_2.0 + randnum(-5.0, 5.0) as f32,
                  randnum(86.0, 90.0) as f32,
                );

                sleep!(randnum(250, 350));

                bot.swing_arm();
                bot.start_use_item();

                sleep!(randnum(150, 250));

                bot.set_direction(original_direction_1.0, original_direction_1.1);

                setmst(&index, StateName::Looking, false).await;
                setmst(&index, StateName::Interacting, false).await;
              }
            }
          }
          "capsule" => {
            let position = bot.feet_pos();

            let block_positions = vec![
              BlockPos {
                x: (position.x - 1.0).floor() as i32,
                y: position.y.floor() as i32,
                z: position.z.floor() as i32,
              },
              BlockPos {
                x: (position.x + 1.0).floor() as i32,
                y: position.y.floor() as i32,
                z: position.z.floor() as i32,
              },
              BlockPos {
                x: position.x.floor() as i32,
                y: position.y.floor() as i32,
                z: (position.z - 1.0).floor() as i32,
              },
              BlockPos {
                x: position.x.floor() as i32,
                y: position.y.floor() as i32,
                z: (position.z + 1.0).floor() as i32,
              },
              BlockPos {
                x: (position.x - 1.0).floor() as i32,
                y: (position.y + 1.0).floor() as i32,
                z: position.z.floor() as i32,
              },
              BlockPos {
                x: (position.x + 1.0).floor() as i32,
                y: (position.y + 1.0).floor() as i32,
                z: position.z.floor() as i32,
              },
              BlockPos {
                x: position.x.floor() as i32,
                y: (position.y + 1.0).floor() as i32,
                z: (position.z - 1.0).floor() as i32,
              },
              BlockPos {
                x: position.x.floor() as i32,
                y: (position.y + 1.0).floor() as i32,
                z: (position.z + 1.0).floor() as i32,
              },
              BlockPos {
                x: (position.x - 1.0).floor() as i32,
                y: (position.y + 2.0).floor() as i32,
                z: position.z.floor() as i32,
              },
              BlockPos {
                x: position.x.floor() as i32,
                y: (position.y + 2.0).floor() as i32,
                z: position.z.floor() as i32,
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

              if let Some(slot) = block_slot {
                if getst(&index, State::CanLooking).await && getst(&index, State::CanInteracting).await {
                  setmst(&index, StateName::Looking, true).await;
                  setmst(&index, StateName::Interacting, true).await;

                  bot.take_item(&index, slot, true).await;

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

                  bot.look_at(Vec3::new(pos.x as f64, pos.y as f64, pos.z as f64));

                  sleep!(randnum(50, 100));

                  bot.swing_arm();
                  bot.start_use_item();

                  if count == 10 {
                    bot.set_crouching(false);
                  }

                  setmst(&index, StateName::Looking, false).await;
                  setmst(&index, StateName::Interacting, false).await;

                  sleep!(randnum(100, 150));
                }
              }
            }
          }
          "unite" => {
            let mut positions = vec![];

            for username in PROFILE_SYSTEM.get_all_connected().await.keys() {
              let Some(b) = REGISTRY_SYSTEM.get_bot(username) else {
                continue;
              };

              positions.push(b.feet_pos());
            }

            let average_cords = get_average_coordinates_of_bots(&positions);

            go_to(index, average_cords.0 as i32, average_cords.2 as i32);
          }
          "turn" => {
            let direction = bot.direction();
            bot.set_direction(direction.0 - 90.0, direction.1);
          }
          "zero" => {
            bot.set_direction(0.0, 0.0);
          }
          "form-circle" => {
            let mut positions = vec![];

            for username in PROFILE_SYSTEM.get_all_connected().await.keys() {
              let Some(b) = REGISTRY_SYSTEM.get_bot(username) else {
                continue;
              };

              positions.push(b.feet_pos());
            }

            let average_cords = get_average_coordinates_of_bots(&positions);

            let angle = 2.0 * PI * (number as f32) / (PROFILE_SYSTEM.get_connected_count().await as f32);
            let x = average_cords.0 + positions.len() as f64 * 0.5 * angle.cos() as f64;
            let z = average_cords.2 + positions.len() as f64 * 0.5 * angle.sin() as f64;

            go_to(index, x as i32, z as i32);
          }
          "form-line" => {
            let mut positions = vec![];

            for username in PROFILE_SYSTEM.get_all_connected().await.keys() {
              let Some(b) = REGISTRY_SYSTEM.get_bot(username) else {
                continue;
              };

              positions.push(b.feet_pos());
            }

            let average_cords = get_average_coordinates_of_bots(&positions);

            let x = average_cords.0 + 1.0 * (number as f64 * 1.0);
            let z = average_cords.2 * (number as f64 * 1.0);

            go_to(index, x as i32, z as i32);
          }
          "pathfinder-stop" => {
            bot.stop_pathfinding();
          }
          _ => {}
        });
      });
    }
  }
}
