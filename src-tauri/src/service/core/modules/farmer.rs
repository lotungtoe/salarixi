use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::registry::builtin::{BlockKind, ItemKind};
use azalea::Vec3;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::common::get_block_state;
use crate::service::core::controllers::{getst, gettskact, killtsk, pushrtsk, setmst, State, StateName, TaskName};
use crate::service::core::extensions::{BotDefaultExt, BotInventoryExt, BotRotationExt};
use crate::service::core::traits::SalarixiModule;
use crate::service::tools::*;
use crate::take_bot;

struct Tool {
  slot: Option<usize>,
  priority: u8,
}

#[derive(Clone, Deserialize)]
pub struct FarmerModule;

#[derive(Clone, Deserialize)]
pub struct FarmerOptions {
  pub mode: String,
  pub delay: Option<u64>,
  pub state: bool,
}

impl FarmerModule {
  async fn auto_tool(bot: &Client) {
    let mut tools = vec![];

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if !item.is_empty() {
          match item.kind() {
            ItemKind::WoodenHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 0,
              });
            }
            ItemKind::GoldenHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 1,
              });
            }
            ItemKind::StoneHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 2,
              });
            }
            ItemKind::CopperHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 3,
              });
            }
            ItemKind::IronHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 4,
              });
            }
            ItemKind::DiamondHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 5,
              });
            }
            ItemKind::NetheriteHoe => {
              tools.push(Tool {
                slot: Some(slot),
                priority: 6,
              });
            }
            _ => {}
          }
        }
      }
    }

    let mut best_tool = Tool {
      slot: None,
      priority: 0,
    };

    for w in tools {
      if w.priority > best_tool.priority {
        best_tool = w;
      }
    }

    if let Some(slot) = best_tool.slot {
      bot.take_item(slot, true).await;
    }
  }

  fn this_is_grown_plant(id: u16) -> bool {
    return vec![10464, 5117, 14612, 10472].contains(&id);
  }

  fn this_is_plant(kind: BlockKind) -> bool {
    return vec![
      BlockKind::Potatoes,
      BlockKind::Carrots,
      BlockKind::Wheat,
      BlockKind::Beetroots,
    ]
    .contains(&kind);
  }

  fn this_is_farmland_without_plant(bot: &Client, kind: BlockKind, block_pos: BlockPos) -> bool {
    if kind == BlockKind::Farmland {
      let block_above = BlockPos::new(block_pos.x, block_pos.y + 1, block_pos.z);

      if let Some(state) = get_block_state(bot, block_above) {
        if state.is_air() {
          return true;
        }
      }
    }

    false
  }

  async fn fertilize_plant(bot: &Client, mode: &String) {
    let mut ferilizer_slot = None;

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if !item.is_empty() {
          if item.kind() == ItemKind::BoneMeal {
            ferilizer_slot = Some(slot);
            break;
          }
        }
      }
    }

    if let Some(slot) = ferilizer_slot {
      bot.take_item(slot, true).await;

      for _ in 0..=4 {
        sleep(Duration::from_millis(Self::generate_delay(mode))).await;
        bot.start_use_item();
      }
    }
  }

  async fn take_plant(bot: &Client) -> bool {
    let mut plant_slot = None;

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if !item.is_empty() && plant_slot.is_none() {
          match item.kind() {
            ItemKind::Potato => plant_slot = Some(slot),
            ItemKind::Carrot => plant_slot = Some(slot),
            ItemKind::BeetrootSeeds => plant_slot = Some(slot),
            ItemKind::WheatSeeds => plant_slot = Some(slot),
            _ => {}
          }
        }
      }
    }

    if let Some(slot) = plant_slot {
      bot.take_item(slot, true).await;
      return true;
    }

    false
  }

  fn generate_delay(mode: &String) -> u64 {
    if mode.as_str() == "normal" {
      return randnum(100, 150);
    } else if mode.as_str() == "quick" {
      return 50;
    } else {
      return randnum(20, 30);
    }
  }

  async fn plow_block(bot: &Client, block_pos: BlockPos, mode: &String) {
    if let Some(state) = get_block_state(bot, block_pos) {
      Self::auto_tool(bot).await;
      sleep(Duration::from_millis(50)).await;

      let kind = BlockKind::from(state);

      if kind == BlockKind::CoarseDirt || kind == BlockKind::RootedDirt {
        bot.start_use_item();
        sleep(Duration::from_millis(Self::generate_delay(mode))).await;
        bot.start_use_item();
      } else {
        bot.start_use_item();
      }
    }
  }

  fn block_can_plowed(bot: &Client, kind: BlockKind, block_pos: BlockPos) -> bool {
    if let Some(state) = get_block_state(bot, BlockPos::new(block_pos.x, block_pos.y + 1, block_pos.z)) {
      return vec![
        BlockKind::Dirt,
        BlockKind::DirtPath,
        BlockKind::RootedDirt,
        BlockKind::CoarseDirt,
      ]
      .contains(&kind)
        && state.is_air();
    }

    false
  }

  async fn interact_with_block(bot: &Client, block_pos: BlockPos, options: &FarmerOptions) {
    let username = bot.username();

    if !getst(&username, State::IsEating).await
      && !getst(&username, State::IsDrinking).await
      && getst(&username, State::CanInteracting).await
    {
      if let Some(state) = get_block_state(bot, block_pos) {
        setmst(&username, StateName::Interacting, true).await;

        let kind = BlockKind::from(state);

        if Self::this_is_farmland_without_plant(bot, kind, block_pos) {
          bot.look_at_block(block_pos, true).await;

          if options.mode.as_str() != "ultra" {
            sleep(Duration::from_millis(Self::generate_delay(&options.mode))).await;
          }

          if Self::take_plant(bot).await {
            sleep(Duration::from_millis(Self::generate_delay(&options.mode))).await;
            bot.start_use_item();
            sleep(Duration::from_millis(Self::generate_delay(&options.mode))).await;
          }
        } else {
          if Self::this_is_plant(kind) {
            bot.look_at_block(block_pos, true).await;

            sleep(Duration::from_millis(Self::generate_delay(&options.mode))).await;

            if Self::this_is_grown_plant(state.id()) {
              Self::auto_tool(bot).await;
              sleep(Duration::from_millis(Self::generate_delay(&options.mode))).await;
              bot.mine(block_pos).await;
              sleep(Duration::from_millis(50)).await;
            } else {
              Self::fertilize_plant(bot, &options.mode).await;

              if let Some(new_state) = get_block_state(bot, block_pos) {
                sleep(Duration::from_millis(50)).await;

                if Self::this_is_grown_plant(new_state.id()) {
                  Self::auto_tool(bot).await;
                  sleep(Duration::from_millis(Self::generate_delay(&options.mode))).await;
                  bot.mine(block_pos).await;
                  sleep(Duration::from_millis(50)).await;
                }
              }
            }

            if options.mode.as_str() != "ultra" {
              sleep(Duration::from_millis(Self::generate_delay(&options.mode))).await;
            }
          } else {
            if Self::block_can_plowed(bot, kind, block_pos) {
              bot.look_at_block(block_pos, true).await;
              Self::plow_block(bot, block_pos, &options.mode).await;
            }
          }
        }

        setmst(&username, StateName::Interacting, false).await;
      }
    }
  }

  async fn start(bot: &Client, options: &FarmerOptions) {
    let username = bot.username();

    loop {
      setmst(&username, StateName::Looking, true).await;

      for y in -1..=1 {
        let pos = bot.feet_pos();

        let block_pos = BlockPos::from(Vec3::new(pos.x, pos.y + y as f64, pos.z));

        Self::interact_with_block(bot, block_pos, &options).await;
      }

      for x in -1..=1 {
        for y in -1..=1 {
          for z in -1..=1 {
            let pos = bot.feet_pos();

            let block_pos = BlockPos::from(Vec3::new(pos.x + x as f64, pos.y + y as f64, pos.z + z as f64));

            if block_pos != BlockPos::from(Vec3::new(pos.x, pos.y + y as f64, pos.z)) {
              Self::interact_with_block(bot, block_pos, &options).await;
            }
          }
        }
      }

      sleep(Duration::from_millis(options.delay.unwrap_or(100))).await;

      setmst(&username, StateName::Looking, false).await;
    }
  }
}

impl SalarixiModule<FarmerOptions> for FarmerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, username: String, options: FarmerOptions) -> bool {
    if options.state && !gettskact(&username, TaskName::Farmer).await {
      let username_clone = username.clone();
      let task_handle = tokio::spawn(async move {
        take_bot!(&username_clone, async |bot| Self::start(bot, &options).await);
      });

      pushrtsk(&username, TaskName::Farmer, task_handle).await;
    } else if !options.state && gettskact(&username, TaskName::Farmer).await {
      killtsk(&username, TaskName::Farmer).await;
      setmst(&username, StateName::Looking, false).await;
      setmst(&username, StateName::Interacting, false).await;
    } else {
      return false;
    }

    true
  }
}
