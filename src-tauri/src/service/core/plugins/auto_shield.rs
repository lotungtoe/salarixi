use azalea::ecs::entity::Entity;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;
use std::time::Duration;
use tokio::time::sleep;

use crate::service::core::controllers::{get_state, set_mutual_states, State, StateName};
use crate::service::core::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, EntityType};
use crate::service::core::plugin_manager::PluginName;
use crate::service::core::traits::SalarixiPlugin;
use crate::service::core::{process_is_active, PLUGINS};
use crate::service::tools::*;
use crate::take_bot;

pub struct AutoShieldPlugin;

impl AutoShieldPlugin {
  /// Метод обнаружения опасной сущности и защиты
  async fn defend(&self, bot: &Client) {
    let Some(entity) = self.get_nearest_dangerous_entity(bot) else {
      return;
    };

    let username = bot.username();

    if let Some(menu) = bot.get_inventory_menu() {
      if let Some(item) = menu.slot(45) {
        if item.is_empty() || item.kind() == ItemKind::Shield {
          if get_state(&username, State::IsEating) || get_state(&username, State::IsDrinking) {
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

    if get_state(&username, State::CanLooking) && get_state(&username, State::CanInteracting) {
      set_mutual_states(&username, StateName::Looking, true);
      set_mutual_states(&username, StateName::Interacting, true);

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

      set_mutual_states(&username, StateName::Looking, false);
      set_mutual_states(&username, StateName::Interacting, false);
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
        if !process_is_active() {
          break;
        }

        take_bot!(&username_clone, async |bot| {
          if !bot.workable() {
            return;
          }

          self.defend(bot).await;
        });

        sleep(Duration::from_millis(50)).await;
      }
    });

    PLUGINS.push_root_task(&username, PluginName::AutoShieldPlugin, task);
  }
}
