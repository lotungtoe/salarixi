use std::time::Duration;

use azalea::ecs::entity::Entity;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use tokio::time::sleep;

use crate::bot::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, EntityType};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::{getst, setmst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::other::tools::*;
use crate::take_bot;

pub struct AutoShieldPlugin;

impl AutoShieldPlugin {
  /// Метод обработки тика
  async fn tick(&self, bot: &Client) {
    let Some(entity) = self.get_nearest_dangerous_entity(bot) else {
      return;
    };

    let username = bot.username();

    if let Some(menu) = bot.get_inventory_menu() {
      if let Some(item) = menu.slot(45) {
        if item.is_empty() || item.kind() == ItemKind::Shield {
          if getst(&username, State::IsEating).await || getst(&username, State::IsDrinking).await {
            return;
          }

          let mut shield_equipped = false;

          if item.kind() != ItemKind::Shield {
            for (slot, item) in menu.slots().iter().enumerate() {
              if slot != 45 {
                if item.kind() == ItemKind::Shield {
                  bot.inventory_move_item(ItemKind::Shield, slot, 45, true).await;
                  shield_equipped = true;
                  break;
                }
              }
            }
          } else {
            shield_equipped = true;
          }

          if shield_equipped {
            self.start_defending(bot, entity).await;
          }
        }
      }
    }
  }

  /// Метод получения ближайшей опасной сущности
  fn get_nearest_dangerous_entity(&self, bot: &Client) -> Option<Entity> {
    let mut nearest_entity = None;

    if let Some(nearest_player) = bot.find_nearest_entity(EntityType::Player, 8.0) {
      nearest_entity = Some(nearest_player);
    } else {
      if let Some(nearest_monster) = bot.find_nearest_entity(EntityType::Monster, 8.0) {
        nearest_entity = Some(nearest_monster);
      }
    }

    nearest_entity
  }

  /// Метод прикрытия щитом
  async fn start_defending(&self, bot: &Client, entity: Entity) {
    let username = bot.username();

    if getst(&username, State::CanLooking).await && getst(&username, State::CanInteracting).await {
      setmst(&username, StateName::Looking, true).await;
      setmst(&username, StateName::Interacting, true).await;

      bot.start_use_held_item(InteractionHand::OffHand);

      sleep(Duration::from_millis(50)).await;

      bot.look_at(bot.get_entity_position(entity));

      sleep(Duration::from_millis(randnum(50, 100))).await;

      for _ in 0..=randnum(2, 4) {
        if let Some(e) = self.get_nearest_dangerous_entity(bot) {
          bot.look_at(bot.get_entity_position(e));
        }

        sleep(Duration::from_millis(randnum(50, 100))).await;
      }

      bot.release_use_held_item();

      setmst(&username, StateName::Looking, false).await;
      setmst(&username, StateName::Interacting, false).await;
    }
  }
}

impl SalarixiPlugin for AutoShieldPlugin {
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

        sleep(Duration::from_millis(100)).await;
      }
    });

    PLUGINS.push_task(&username, PluginName::AutoShield, task);
  }
}
