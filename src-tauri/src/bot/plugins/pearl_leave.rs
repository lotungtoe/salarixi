use std::time::Duration;

use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use tokio::time::sleep;

use crate::bot::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::{getst, setmst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::other::tools::randnum;
use crate::take_bot;

pub struct PearlLeavePlugin;

impl PearlLeavePlugin {
  /// Метод обработки тика
  async fn tick(&self, bot: &Client) {
    let health = bot.get_health();

    if health < 6.0 || health > 10.0 {
      return;
    }

    if !self.take_pearl(bot).await {
      return;
    }

    self.throw_pearl(bot).await;
  }

  /// Метод взятия жемчуга в руку
  async fn take_pearl(&self, bot: &Client) -> bool {
    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        if item.kind() != ItemKind::EnderPearl {
          continue;
        }

        bot.take_item(slot, true).await;

        return true;
      }
    }

    false
  }

  /// Метод броска жемчуга
  async fn throw_pearl(&self, bot: &Client) {
    let username = bot.username();

    if !getst(&username, State::CanLooking).await {
      return;
    }

    setmst(&username, StateName::Looking, true).await;

    let y_rot = randnum(-180.0, 180.0);
    let x_rot = randnum(-35.0, -30.0);
    bot.set_direction(y_rot, x_rot);

    setmst(&username, StateName::Looking, false).await;

    tokio::time::sleep(Duration::from_millis(randnum(100, 150))).await;

    bot.jump();

    tokio::time::sleep(Duration::from_millis(randnum(100, 150))).await;

    if !getst(&username, State::CanInteracting).await {
      return;
    }

    setmst(&username, StateName::Interacting, true).await;
    bot.start_use_held_item(InteractionHand::MainHand);
    setmst(&username, StateName::Interacting, false).await;

    tokio::time::sleep(Duration::from_secs(8)).await;
  }
}

impl SalarixiPlugin for PearlLeavePlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, username: String) {
    let username_clone = username.clone();
    let task = tokio::spawn(async move {
      loop {
        take_bot!(&username_clone, async |bot| {
          if !bot.workable() {
            return;
          }

          self.tick(bot).await;
        });

        sleep(Duration::from_millis(200)).await;
      }
    });

    PLUGINS.push_task(&username, PluginName::PearlLeave, task);
  }
}
