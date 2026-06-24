use azalea::container::ContainerHandle;
use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::Vec3;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::common::get_block_state;
use crate::service::core::controllers::push_root_task;
use crate::service::core::controllers::State;
use crate::service::core::controllers::StateName;
use crate::service::core::controllers::TaskName;
use crate::service::core::controllers::TASKS;
use crate::service::core::controllers::{get_state, kill_task, set_mutual_states, set_state};
use crate::service::core::extensions::BotMovementExt;
use crate::service::core::traits::SalarixiModule;
use crate::service::tools::*;
use crate::take_bot;

pub struct StealerModule;

#[derive(Deserialize)]
pub struct StealerOptions {
  pub target: String,
  pub radius: Option<i32>,
  pub delay: Option<u64>,
  pub state: bool,
}

impl StealerModule {
  pub fn new() -> Self {
    Self
  }

  fn check_block_id(block_id: u16, target: &String) -> bool {
    match target.as_str() {
      "chest" => {
        return vec![3793, 3787, 3805, 3799].contains(&block_id);
      }
      "barrel" => {
        return vec![20547, 20543, 20541, 20549, 20545].contains(&block_id);
      }
      "shulker" => {
        return vec![
          14666, 14672, 14678, 14720, 14756, 14714, 14762, 14744, 14702, 14696, 14708, 14750, 14684, 14726, 14738,
          14732,
        ]
        .contains(&block_id);
      }
      _ => {}
    }

    false
  }

  fn find_nearest_targets(bot: &Client, center: Vec3, target: &String, radius: i32) -> Vec<azalea::BlockPos> {
    let mut positions = Vec::new();

    for x in -radius..=radius {
      for y in -radius..=radius {
        for z in -radius..=radius {
          let block_pos = BlockPos::new(
            (center.x as i32 + x) as i32,
            (center.y as i32 + y) as i32,
            (center.z as i32 + z) as i32,
          );

          if let Some(state) = get_block_state(bot, block_pos) {
            if Self::check_block_id(state.id(), target) {
              positions.push(block_pos);
            }
          }
        }
      }
    }

    positions
  }

  async fn extract_all_items(bot: &Client, container: &ContainerHandle) {
    if let Some(menu) = container.menu() {
      bot.freeze_move();

      let username = bot.username();

      set_state(&username, State::CanAttacking, false);
      set_state(&username, State::CanInteracting, false);
      set_state(&username, State::CanEating, false);
      set_state(&username, State::CanDrinking, false);

      for slot in 0..=26 {
        if let Some(item) = menu.slot(slot) {
          if !item.is_empty() {
            container.shift_click(slot);
          }
        }
      }

      bot.unfreeze_move();

      set_state(&username, State::CanAttacking, true);
      set_state(&username, State::CanInteracting, true);
      set_state(&username, State::CanEating, true);
      set_state(&username, State::CanDrinking, true);
    }
  }

  async fn start(bot: &Client, options: &StealerOptions) {
    let username = bot.username();

    loop {
      let position = bot.position();
      let direction = bot.direction();

      let target_positions = Self::find_nearest_targets(bot, position, &options.target, options.radius.unwrap_or(5));

      for pos in target_positions {
        if get_state(&username, State::CanLooking) && get_state(&username, State::CanInteracting) {
          set_mutual_states(&username, StateName::Looking, true);
          set_mutual_states(&username, StateName::Interacting, true);

          bot.look_at(pos.center());

          sleep(Duration::from_millis(randnum(50, 100))).await;

          if let Some(container) = bot.open_container_at(pos).await {
            Self::extract_all_items(bot, &container).await;
            container.close();
            sleep(Duration::from_millis(randnum(200, 350))).await;
          }

          set_mutual_states(&username, StateName::Looking, false);
          set_mutual_states(&username, StateName::Interacting, false);
        }
      }

      if get_state(&username, State::CanLooking) {
        bot.set_direction(
          direction.0 + randnum(-2.5, 2.5) as f32,
          direction.1 + randnum(-2.5, 2.5) as f32,
        );
      }

      sleep(Duration::from_millis(options.delay.unwrap_or(1000))).await;
    }
  }
}

impl SalarixiModule<StealerOptions> for StealerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, username: String, options: StealerOptions) -> bool {
    if options.state && !TASKS.get_task_activity(&username, TaskName::Stealer) {
      let username_clone = username.clone();
      let task_handle = tokio::spawn(async move {
        take_bot!(&username_clone, async |bot| Self::start(bot, &options).await);
      });

      push_root_task(&username, TaskName::Stealer, task_handle);
    } else if !options.state && TASKS.get_task_activity(&username, TaskName::Stealer) {
      kill_task(&username, TaskName::Stealer);
    } else {
      return false;
    }

    true
  }
}
