use std::time::Duration;

use azalea::auto_tool::best_tool_in_hotbar_for_block;
use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::Vec3;
use azalea::WalkDirection;
use serde::Deserialize;
use tokio::time::sleep;

use crate::bot::common::get_block_state;
use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotMovementExt};
use crate::bot::systems::{gettskact, killtsk, pushrtsk, setmst, StateName, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::other::tools::*;
use crate::take_bot;

pub struct MinerModule;

#[derive(Deserialize)]
pub struct MinerOptions {
  pub mode: String,
  pub tunnel: String,
  pub look: String,
  pub direction_x: Option<f32>,
  pub slot: Option<u8>,
  pub delay: Option<u64>,
  pub state: bool,
}

impl MinerModule {
  pub fn new() -> Self {
    Self
  }

  fn is_breakable_block(block_id: u16) -> bool {
    return vec![86, 88, 87, 89, 94, 0, 110, 104, 106, 102, 108].contains(&block_id);
  }

  fn can_reach_block(eye_pos: Vec3, block_pos: BlockPos) -> bool {
    if eye_pos.distance_to(Vec3::new(block_pos.x as f64, block_pos.y as f64, block_pos.z as f64)) < 4.5 {
      return true;
    }

    false
  }

  async fn micro_offset(bot: &Client) {
    if randchance(0.8) {
      let walk_directions = vec![
        WalkDirection::Left,
        WalkDirection::Right,
        WalkDirection::ForwardLeft,
        WalkDirection::ForwardRight,
        WalkDirection::BackwardLeft,
        WalkDirection::BackwardRight,
      ];

      let walk_direction = randelem(walk_directions.as_ref());

      bot.walk(*walk_direction);
      sleep(Duration::from_millis(randnum(150, 250))).await;
      bot.walk(WalkDirection::None);
    }
  }

  async fn look_at_block(bot: &Client, block_pos: BlockPos, look: String) {
    let mut center = block_pos.center();

    if look.as_str() == "smooth" {
      let pre = Vec3::new(
        (block_pos.x as f64) + randnum(-0.05, 0.05),
        (block_pos.y as f64) + randnum(-0.05, 0.05),
        (block_pos.z as f64) + randnum(-0.05, 0.05),
      );

      bot.look_at(pre);

      sleep(Duration::from_millis(randnum(50, 150))).await;
    }

    if randchance(0.7) {
      center.x += randnum(-0.03, 0.03);
    }

    if randchance(0.7) {
      center.y += randnum(-0.03, 0.03);
    }

    if randchance(0.7) {
      center.z += randnum(-0.03, 0.03);
    }

    bot.look_at(center);
  }

  async fn move_forward(bot: &Client, territory: Vec<BlockPos>) {
    for block_pos in territory {
      if let Some(state) = get_block_state(bot, block_pos) {
        if !state.is_air() && Self::is_breakable_block(state.id()) {
          return;
        }
      }
    }

    bot.walk(WalkDirection::Forward);
    sleep(Duration::from_millis(randnum(50, 150))).await;
    bot.walk(WalkDirection::None);
  }

  fn get_territory(pos: Vec3, tunnel: String) -> Vec<BlockPos> {
    let x = pos.x.floor() as i32;
    let y = pos.y.floor() as i32;
    let z = pos.z.floor() as i32;

    let territory;

    match tunnel.as_str() {
      "2x2x2" => {
        territory = vec![
          BlockPos::new(x + 1, y, z),
          BlockPos::new(x - 1, y, z),
          BlockPos::new(x, y, z + 1),
          BlockPos::new(x, y, z - 1),
        ];
      }
      "3x2x2" => {
        territory = vec![
          BlockPos::new(x + 1, y, z),
          BlockPos::new(x - 1, y, z),
          BlockPos::new(x, y, z + 1),
          BlockPos::new(x, y, z - 1),
        ];
      }
      "2x3x3" => {
        territory = vec![
          BlockPos::new(x + 1, y, z),
          BlockPos::new(x - 1, y, z),
          BlockPos::new(x, y, z + 1),
          BlockPos::new(x, y, z - 1),
          BlockPos::new(x + 2, y, z),
          BlockPos::new(x - 2, y, z),
          BlockPos::new(x, y, z + 2),
          BlockPos::new(x, y, z - 2),
        ];
      }
      _ => {
        territory = vec![
          BlockPos::new(x + 1, y, z),
          BlockPos::new(x - 1, y, z),
          BlockPos::new(x, y, z + 1),
          BlockPos::new(x, y, z - 1),
          BlockPos::new(x + 2, y, z),
          BlockPos::new(x - 2, y, z),
          BlockPos::new(x, y, z + 2),
          BlockPos::new(x, y, z - 2),
          BlockPos::new(x + 3, y, z),
          BlockPos::new(x - 3, y, z),
          BlockPos::new(x, y, z + 3),
          BlockPos::new(x, y, z - 3),
        ];
      }
    };

    territory
  }

  async fn default_mine(bot: &Client, options: &MinerOptions) {
    bot.left_click_mine(true);
    bot.walk(WalkDirection::Forward);
    bot.set_direction(options.direction_x.unwrap_or(0.0), 40.0 + randnum(-3.5, 3.5) as f32);
  }

  async fn extended_mine(bot: &Client, options: &MinerOptions) {
    loop {
      let position = bot.feet_pos();

      let territory = Self::get_territory(position, options.tunnel.clone());

      for pos in territory.clone() {
        let heights = match options.tunnel.as_str() {
          "3x2x2" => vec![2, 1, 0],
          "3x3x3" => vec![2, 1, 0],
          _ => vec![1, 0],
        };

        for height in heights {
          let block_pos = BlockPos::new(pos.x, pos.y + height, pos.z);

          if let Some(state) = get_block_state(bot, block_pos) {
            if !state.is_air()
              && Self::is_breakable_block(state.id())
              && Self::can_reach_block(bot.eye_pos(), block_pos)
            {
              let should_shift = randchance(0.2);

              if should_shift {
                bot.set_crouching(true);
              }

              Self::look_at_block(bot, block_pos, options.look.clone()).await;

              sleep(Duration::from_millis(randnum(100, 200))).await;

              if let Some(slot) = options.slot {
                bot.set_selected_hotbar_slot(slot);
              } else {
                if let Some(menu) = bot.get_inventory_menu() {
                  let best_tool = best_tool_in_hotbar_for_block(state, &menu).index;
                  bot.set_selected_hotbar_slot(best_tool as u8);
                }
              }

              if should_shift {
                bot.set_crouching(false);
              }

              Self::micro_offset(bot).await;

              bot.start_mining(block_pos);

              loop {
                if let Some(s) = get_block_state(bot, block_pos) {
                  if state.is_air() || !Self::is_breakable_block(s.id()) {
                    break;
                  }
                } else {
                  break;
                }

                sleep(Duration::from_millis(50)).await;
              }

              Self::micro_offset(bot).await;

              sleep(Duration::from_millis(randnum(50, 100))).await;

              if let Some(s) = get_block_state(bot, block_pos) {
                if !s.is_air() || Self::is_breakable_block(s.id()) {
                  bot.walk(WalkDirection::Backward);
                  sleep(Duration::from_millis(randnum(50, 100))).await;
                  bot.walk(WalkDirection::None);
                }
              }
            }
          }
        }
      }

      let direction = bot.direction();

      if randchance(0.5) {
        bot.set_direction(
          options.direction_x.unwrap_or(0.0) + randnum(-2.5, 2.5) as f32,
          direction.1 + randnum(-2.5, 2.5) as f32,
        );
      } else {
        if randchance(0.3) {
          bot.set_direction(
            options.direction_x.unwrap_or(0.0) + randnum(-1.3, 1.3) as f32,
            direction.1 + randnum(-1.3, 1.3) as f32,
          );
        } else {
          bot.set_direction(options.direction_x.unwrap_or(0.0), direction.1);
        }
      }

      sleep(Duration::from_millis(randnum(50, 100))).await;

      Self::move_forward(bot, territory).await;

      sleep(Duration::from_millis(options.delay.unwrap_or(200))).await;
    }
  }

  async fn start(bot: &Client, options: &MinerOptions) {
    match options.mode.as_str() {
      "default" => Self::default_mine(bot, &options).await,
      "extended" => Self::extended_mine(bot, &options).await,
      _ => {}
    }
  }
}

impl SalarixiModule<MinerOptions> for MinerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, username: String, options: MinerOptions) -> bool {
    if options.state && !gettskact(&username, TaskName::Miner).await {
      let username_clone = username.clone();
      let task_handle = tokio::spawn(async move {
        take_bot!(&username_clone, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&username, TaskName::Miner, task_handle).await;
    } else if !options.state && gettskact(&username, TaskName::Miner).await {
      killtsk(&username, TaskName::Miner).await;

      take_bot!(&username, async |bot| {
        bot.left_click_mine(false);
        bot.set_crouching(false);
        bot.stop_move().await;
      });

      setmst(&username, StateName::Interacting, false).await;
      setmst(&username, StateName::Looking, false).await;
    } else {
      return false;
    }

    true
  }
}
