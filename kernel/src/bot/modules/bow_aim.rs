use azalea::prelude::*;
use azalea::registry::builtin::ItemKind;
use azalea::Vec3;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, EntityType};
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::bot::systems::tasks::{gettskact, killtsk, pushetsk, pushrtsk, TaskName};
use crate::bot::traits::SalarixiModule;
use crate::tools::*;
use crate::{sleep, take_bot};

#[derive(PartialEq, Index)]
enum Target {
  Player = 0,
  Monster = 1,
  Animal = 2,
  Vehicle = 3,
  Any = 4,
  Custom = 5,
}

impl Target {
  pub fn to_string(&self) -> String {
    let str = match self {
      Self::Player => "player",
      Self::Monster => "monster",
      Self::Animal => "animal",
      Self::Vehicle => "vehicle",
      Self::Any => "any",
      Self::Custom => "custom",
    };

    str.to_string()
  }
}

pub struct BowAimOptions {
  target: Target,
  target_nickname: Option<String>,
  delay: Option<u64>,
  max_distance: Option<f64>,
  state: u8,
}

impl BowAimOptions {
  pub fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      target: Target::from_index(u8::read(buf)?)?,
      target_nickname: Option::read(buf)?,
      delay: Option::read(buf)?,
      max_distance: Option::read(buf)?,
      state: u8::read(buf)?,
    })
  }
}

pub struct BowAimModule;

impl BowAimModule {
  pub fn new() -> Self {
    Self
  }

  fn find_bow_in_inventory(bot: &Client) -> Option<usize> {
    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if item.kind() == ItemKind::Bow {
          return Some(slot);
        }
      }
    }

    None
  }

  async fn aiming(index: u8, target: String, distance: f64) {
    let extra_task = tokio::spawn(async move {
      loop {
        if !getst(&index, State::CanLooking).await {
          sleep!(200);
          continue;
        }

        let mut nearest_entity_exist = false;

        take_bot!(&index, async |bot| {
          if let Some(entity) = bot.find_nearest_entity(EntityType::from(target.clone()), distance) {
            nearest_entity_exist = true;

            let dest_pos = bot.get_entity_position(entity);

            setmst(&index, StateName::Looking, true).await;
            sleep!(randnum(50, 100));
            bot.look_at(Vec3::new(
              dest_pos.x,
              dest_pos.y + bot.get_entity_eye_height(entity),
              dest_pos.z,
            ));
            sleep!(randnum(50, 100));
            setmst(&index, StateName::Looking, false).await;
          }
        });

        if !nearest_entity_exist {
          sleep!(200);
        }
      }
    });

    pushetsk(&index, TaskName::BowAim, extra_task).await;
  }

  async fn shoot(bot: &Client, index: &u8, target: String, distance: f64) {
    if !getst(index, State::CanInteracting).await {
      return;
    }

    let Some(bow_slot) = Self::find_bow_in_inventory(bot) else {
      return;
    };

    let mut bow_picked = false;

    if gettskact(index, TaskName::Killaura).await && bow_slot != 45 {
      if let Some(item) = bot.get_item_by_slot(45) {
        if item.is_empty() {
          bot.inventory_swap_click(index, bow_slot, 45, true).await;
          bow_picked = true;
        }
      }
    }

    if !bow_picked {
      bot.take_item(index, bow_slot, true).await;
    }

    setmst(index, StateName::Interacting, true).await;

    bot.start_use_item();

    sleep!(randnum(900, 1100));

    if let Some(entity) = bot.find_nearest_entity(EntityType::from(target), distance) {
      let dest_pos = bot.get_entity_position(entity);
      let distance = bot.eye_pos().distance_to(dest_pos);

      bot.look_at(Vec3::new(dest_pos.x, dest_pos.y + distance * 0.16, dest_pos.z));

      if distance > 50.0 {
        bot.jump();

        sleep!(100);

        let dest_pos = bot.get_entity_position(entity);
        let distance = bot.eye_pos().distance_to(dest_pos);

        bot.look_at(Vec3::new(dest_pos.x, dest_pos.y + distance * 0.12, dest_pos.z));
      }

      sleep!(50);
    }

    bot.release_use_item();

    setmst(index, StateName::Interacting, false).await;
  }

  async fn start(bot: &Client, index: &u8, options: &BowAimOptions) {
    let target = if options.target == Target::Custom {
      options.target_nickname.clone()
    } else {
      Some(options.target.to_string())
    };

    let Some(t) = target else {
      return;
    };

    Self::aiming(*index, t.clone(), options.max_distance.unwrap_or(70.0)).await;

    loop {
      Self::shoot(bot, index, t.clone(), options.max_distance.unwrap_or(70.0)).await;
      sleep!(options.delay.unwrap_or(50));
    }
  }
}

impl SalarixiModule<BowAimOptions> for BowAimModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: std::sync::Arc<BowAimOptions>) -> bool {
    if options.state == 1 && !gettskact(&index, TaskName::BowAim).await {
      let task_handle = tokio::spawn(async move {
        take_bot!(&index, async |bot| Self::start(bot, &index, &options).await);
      });

      pushrtsk(&index, TaskName::BowAim, task_handle).await;
    } else if options.state == 0 && gettskact(&index, TaskName::BowAim).await {
      killtsk(&index, TaskName::BowAim).await;

      take_bot!(&index, async |bot| {
        bot.release_use_item();
      });

      setmst(&index, StateName::Looking, false).await;
      setmst(&index, StateName::Interacting, false).await;
    } else {
      return false;
    }

    true
  }
}
