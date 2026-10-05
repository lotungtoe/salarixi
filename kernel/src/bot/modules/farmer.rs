use azalea::core::position::BlockPos;
use azalea::prelude::*;
use azalea::registry::builtin::{BlockKind, ItemKind};
use azalea::Vec3;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::common::get_block_state;
use crate::bot::extensions::{BotDefaultExt, BotInventoryExt, BotRotationExt};
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushrtsk, TaskName};
use crate::bot::traits::{ModuleOptions, SalarixiModule};
use crate::tools::*;
use crate::{sleep, take_bot};

struct Tool {
  slot: Option<usize>,
  priority: u8,
}

#[derive(PartialEq, Index)]
enum Mode {
  Normal = 0,
  Quick = 1,
  Ultra = 2,
}

#[derive(PartialEq)]
enum Plant {
  Potato,
  Carrot,
  WheatSeeds,
  BeetrootSeeds,
}

impl Plant {
  fn from_str(value: &str) -> Option<Self> {
    match value {
      "potato" | "potatoes" => Some(Self::Potato),
      "carrot" | "carrots" => Some(Self::Carrot),
      "wheat" => Some(Self::WheatSeeds),
      "beetroot" => Some(Self::BeetrootSeeds),
      _ => None,
    }
  }

  fn from_item_kind(item_kind: &ItemKind) -> Option<Self> {
    match item_kind {
      ItemKind::Potato => Some(Self::Potato),
      ItemKind::Carrot => Some(Self::Carrot),
      ItemKind::WheatSeeds => Some(Self::WheatSeeds),
      ItemKind::BeetrootSeeds => Some(Self::BeetrootSeeds),
      _ => None,
    }
  }

  fn from_block_kind(block_kind: &BlockKind) -> Option<Self> {
    match block_kind {
      BlockKind::Potatoes => Some(Self::Potato),
      BlockKind::Carrots => Some(Self::Carrot),
      BlockKind::Wheat => Some(Self::WheatSeeds),
      BlockKind::Beetroots => Some(Self::BeetrootSeeds),
      _ => None,
    }
  }
}

#[derive(PartialEq)]
enum Surface {
  GrassBlock,
  Dirt,
  DirtPath,
  RootedDirt,
  CoarseDirt,
}

impl Surface {
  fn from_str(value: &str) -> Option<Self> {
    match value {
      "grass_block" => Some(Self::GrassBlock),
      "dirt" => Some(Self::Dirt),
      "dirt_path" => Some(Self::DirtPath),
      "rooted_dirt" => Some(Self::RootedDirt),
      "coarse_dirt" => Some(Self::CoarseDirt),
      _ => None,
    }
  }

  fn from_block_kind(block_kind: &BlockKind) -> Option<Self> {
    match block_kind {
      BlockKind::GrassBlock => Some(Self::GrassBlock),
      BlockKind::Dirt => Some(Self::Dirt),
      BlockKind::DirtPath => Some(Self::DirtPath),
      BlockKind::RootedDirt => Some(Self::RootedDirt),
      BlockKind::CoarseDirt => Some(Self::CoarseDirt),
      _ => None,
    }
  }
}

pub struct FarmerOptions {
  mode: Mode,
  delay: Option<u64>,
  tool_slot: Option<u8>,
  target_plants: Vec<Plant>,
  target_surfaces: Vec<Surface>,
  use_auto_tool: bool,
  state: u8,
}

impl ModuleOptions for FarmerOptions {
  fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      mode: Mode::from_index(u8::read(buf)?)?,
      delay: Option::read(buf)?,
      tool_slot: Option::read(buf)?,
      target_plants: {
        let full_string = String::read(buf)?;
        let splitted = full_string.split(',').collect::<Vec<&str>>();
        let mut plants = Vec::new();

        for part in splitted {
          if let Some(plant) = Plant::from_str(part.trim()) {
            plants.push(plant);
          }
        }

        plants
      },
      target_surfaces: {
        let full_string = String::read(buf)?;
        let splitted = full_string.split(',').collect::<Vec<&str>>();
        let mut surfaces = Vec::new();

        for part in splitted {
          if let Some(surface) = Surface::from_str(part.trim()) {
            surfaces.push(surface);
          }
        }

        surfaces
      },
      use_auto_tool: bool::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct FarmerModule;

impl FarmerModule {
  async fn auto_tool(bot: &Client, index: &u8) {
    let mut tools = vec![];

    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if item.is_empty() {
          continue;
        }

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
      bot.take_item(index, slot, false).await;
    }
  }

  fn this_is_grown_plant(id: u16) -> bool {
    return vec![10666, 14814, 5318, 10674].contains(&id);
  }

  fn this_is_plant(kind: &BlockKind, target_plants: &Vec<Plant>) -> bool {
    if let Some(plant) = Plant::from_block_kind(kind) {
      if target_plants.contains(&plant) {
        return true;
      }
    }

    false
  }

  fn this_is_farmland_without_plant(bot: &Client, kind: BlockKind, block_pos: BlockPos) -> bool {
    if kind != BlockKind::Farmland {
      return false;
    }

    let block_above = BlockPos::new(block_pos.x, block_pos.y + 1, block_pos.z);
    get_block_state(bot, block_above).map(|s| s.is_air()).unwrap_or(false)
  }

  async fn fertilize_plant(bot: &Client, index: &u8, mode: &Mode, block_pos: BlockPos) {
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
      bot.look_at_block(block_pos, false).await;
      bot.take_item(index, slot, false).await;

      for _ in 0..=4 {
        sleep!(Self::generate_delay(mode));
        bot.start_use_item();
      }
    }
  }

  async fn take_plant(bot: &Client, index: &u8, target_plants: &Vec<Plant>) -> bool {
    let Some(menu) = bot.get_inventory_menu() else {
      return false;
    };

    for (slot, item) in menu.slots().iter().enumerate() {
      if item.is_empty() {
        continue;
      }

      if let Some(plant) = Plant::from_item_kind(&item.kind()) {
        if target_plants.contains(&plant) {
          bot.take_item(index, slot, false).await;
          return true;
        }
      }
    }

    false
  }

  fn generate_delay(mode: &Mode) -> u64 {
    match mode {
      Mode::Normal => randnum(200, 350),
      Mode::Quick => randnum(100, 150),
      Mode::Ultra => 50,
    }
  }

  async fn plow_block(bot: &Client, kind: BlockKind, mode: &Mode) {
    if kind == BlockKind::CoarseDirt || kind == BlockKind::RootedDirt {
      bot.start_use_item();
      sleep!(Self::generate_delay(mode));
      bot.start_use_item();
    } else {
      bot.start_use_item();
    }
  }

  fn block_can_plowed(kind: &BlockKind, target_surfaces: &Vec<Surface>) -> bool {
    if let Some(surface) = Surface::from_block_kind(kind) {
      return target_surfaces.contains(&surface);
    }

    false
  }

  async fn interact_with_block(bot: &Client, index: &u8, block_pos: BlockPos, options: &FarmerOptions) {
    let Some(state) = get_block_state(bot, block_pos) else {
      return;
    };

    let kind = state.as_block_kind();
    let select_tool = async || {
      if options.use_auto_tool {
        Self::auto_tool(bot, index).await;
      } else {
        if let Some(slot) = options.tool_slot {
          bot.set_selected_hotbar_slot(slot);
        }
      }
    };

    if Self::this_is_farmland_without_plant(bot, kind, block_pos) {
      if Self::take_plant(bot, index, &options.target_plants).await {
        bot.look_at_block(block_pos, false).await;
        sleep!(Self::generate_delay(&options.mode));
        bot.start_use_item();
        sleep!(Self::generate_delay(&options.mode));
      }
    } else {
      if Self::this_is_plant(&kind, &options.target_plants) {
        if Self::this_is_grown_plant(state.id()) {
          bot.look_at_block(block_pos, false).await;
          select_tool().await;
          sleep!(Self::generate_delay(&options.mode));
          bot.mine(block_pos).await;
          sleep!(50);
        } else {
          Self::fertilize_plant(bot, index, &options.mode, block_pos).await;
        }

        if options.mode != Mode::Ultra {
          sleep!(Self::generate_delay(&options.mode));
        }
      } else if Self::block_can_plowed(&kind, &options.target_surfaces) {
        bot.look_at_block(block_pos, false).await;
        select_tool().await;
        Self::plow_block(bot, kind, &options.mode).await;
      }
    }
  }

  async fn start(bot: &Client, index: &u8, options: &FarmerOptions) {
    let delay = options.delay.unwrap_or(100);
    let update_states = async |c: bool| {
      setst(index, State::CanAttacking, !c).await;
      setst(index, State::CanDrinking, !c).await;
      setst(index, State::CanEating, !c).await;
      setmst(index, StateName::Interacting, c).await;
      setmst(index, StateName::Looking, c).await;
    };

    loop {
      for x in -1..=1 {
        for y in -1..=1 {
          for z in -1..=1 {
            if !getst(index, State::CanInteracting).await || !getst(index, State::CanLooking).await {
              continue;
            }

            let Some(foot_pos) = bot.foot_pos() else {
              continue;
            };

            let block_pos = BlockPos::from(Vec3::new(
              foot_pos.x + x as f64,
              foot_pos.y + y as f64,
              foot_pos.z + z as f64,
            ));

            update_states(true).await;
            Self::interact_with_block(bot, index, block_pos, &options).await;
            update_states(false).await;
          }
        }
      }

      sleep!(delay);
    }
  }
}

impl SalarixiModule<FarmerOptions> for FarmerModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<FarmerOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::Farmer).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::Farmer, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::Farmer).await {
      killtsk(&index, TaskName::Farmer).await;
      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;
      setst(&index, State::CanAttacking, true).await;
      setst(&index, State::CanDrinking, true).await;
      setst(&index, State::CanEating, true).await;
    } else {
      return false;
    }

    true
  }
}
