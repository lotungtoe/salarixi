use azalea::ecs::entity::Entity;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;

use crate::bot::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, BotRotationExt, EntityType};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::{getst, setmst, setst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::{sleep, take_bot};

pub struct AutoShieldPlugin;

impl AutoShieldPlugin {
  /// Метод обработки тика
  async fn tick(&self, username: &str) {
    let mut danger_entity = None;

    take_bot!(username, async |bot| {
      danger_entity = self.get_nearest_dangerous_entity(bot);
    });

    if danger_entity.is_none() {
      return;
    }

    self.start_defending(username).await;
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

  /// Метод экипировки щита
  async fn equip_shield(&self, username: &str) -> bool {
    let mut shield_equipped = false;

    take_bot!(username, async |bot| {
      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      let Some(offhand_item) = menu.slot(45) else {
        return;
      };

      if !offhand_item.is_empty() && offhand_item.kind() != ItemKind::Shield {
        return;
      }

      if offhand_item.kind() != ItemKind::Shield {
        for (slot, item) in menu.slots().iter().enumerate() {
          if slot == 45 || item.kind() != ItemKind::Shield {
            continue;
          }

          bot.inventory_move_item(ItemKind::Shield, slot, 45, true).await;
          shield_equipped = true;

          break;
        }
      } else {
        shield_equipped = true;
      }
    });

    shield_equipped
  }

  /// Метод прикрытия щитом
  async fn start_defending(&self, username: &str) {
    if !getst(username, State::CanLooking).await || !getst(username, State::CanInteracting).await {
      return;
    }

    if !self.equip_shield(username).await {
      return;
    }

    setmst(username, StateName::Looking, true).await;
    setmst(username, StateName::Interacting, true).await;
    setst(username, State::CanAttacking, false).await;
    setst(username, State::CanEating, false).await;
    setst(username, State::CanDrinking, false).await;

    take_bot!(username, async |bot| {
      bot.start_use_held_item(InteractionHand::OffHand);
    });

    let mut danger_entity_exists = true;

    while danger_entity_exists {
      if !self.equip_shield(username).await {
        break;
      }

      take_bot!(username, async |bot| {
        if let Some(e) = self.get_nearest_dangerous_entity(bot) {
          bot.look_at_entity(e, true);
        } else {
          danger_entity_exists = false;
        }
      });

      sleep!(50);
    }

    take_bot!(username, async |bot| {
      bot.release_use_held_item();
    });

    setmst(username, StateName::Looking, false).await;
    setmst(username, StateName::Interacting, false).await;
    setst(username, State::CanAttacking, true).await;
    setst(username, State::CanEating, true).await;
    setst(username, State::CanDrinking, true).await;
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
        self.tick(&username_clone).await;
        sleep!(100);
      }
    });

    PLUGINS.push_task(&username, PluginName::AutoShield, task);
  }
}
