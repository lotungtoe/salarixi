use std::sync::Arc;

use azalea::ecs::entity::Entity;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::registry::builtin::ItemKind;

use crate::bot::extensions::{BotDefaultExt, BotInteractExt, BotInventoryExt, BotRotationExt};
use crate::bot::plugin_manager::PluginName;
use crate::bot::systems::states::{getst, setmst, setst, State, StateName};
use crate::bot::traits::SalarixiPlugin;
use crate::bot::PLUGINS;
use crate::launch::options::AutoShieldOptions;
use crate::{sleep, take_bot};

pub struct AutoShieldPlugin;

impl AutoShieldPlugin {
  /// Метод обработки тика
  async fn tick(&self, index: &u8, options: &AutoShieldOptions) {
    let mut danger_entity = None;

    take_bot!(index, async |bot| {
      danger_entity = self.get_nearest_dangerous_entity(bot, options);
    });

    if danger_entity.is_none() {
      return;
    }

    self.start_defending(index, options).await;
  }

  /// Метод получения ближайшей опасной сущности
  fn get_nearest_dangerous_entity(&self, bot: &Client, options: &AutoShieldOptions) -> Option<Entity> {
    for entity_filter in &options.dangerous_entities {
      if let Some(entity) = bot.find_nearest_entity(entity_filter, options.distance) {
        return Some(entity);
      }
    }

    None
  }

  /// Метод экипировки щита
  async fn equip_shield(&self, index: &u8, options: &AutoShieldOptions) -> bool {
    let mut shield_equipped = false;

    take_bot!(index, async |bot| {
      let Some(menu) = bot.get_inventory_menu() else {
        return;
      };

      let Some(offhand_item) = menu.slot(45) else {
        return;
      };

      if !options.ignore_item_in_offhand && !offhand_item.is_empty() && offhand_item.kind() != ItemKind::Shield {
        return;
      }

      if offhand_item.kind() != ItemKind::Shield {
        for (slot, item) in menu.slots().iter().enumerate() {
          if slot == 45 || item.kind() != ItemKind::Shield {
            continue;
          }

          bot.inventory_move_item(index, ItemKind::Shield, slot, 45, true).await;
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
  async fn start_defending(&self, index: &u8, options: &AutoShieldOptions) {
    if !getst(index, State::CanLooking).await || !getst(index, State::CanInteracting).await {
      return;
    }

    if !self.equip_shield(index, options).await {
      sleep!(options.delay_after_not_equipped as u64);
      return;
    }

    setst(index, State::CanAttacking, false).await;
    setst(index, State::CanEating, false).await;
    setst(index, State::CanDrinking, false).await;
    setmst(index, StateName::Looking, true).await;
    setmst(index, StateName::Interacting, true).await;

    take_bot!(index, async |bot| {
      bot.start_use_item_by(InteractionHand::OffHand);
    });

    let mut danger_entity_exists = true;

    while danger_entity_exists {
      if !self.equip_shield(index, options).await {
        break;
      }

      take_bot!(index, async |bot| {
        if let Some(e) = self.get_nearest_dangerous_entity(bot, options) {
          bot.look_at_entity(e, true);
        } else {
          danger_entity_exists = false;
        }
      });

      sleep!(50);
    }

    take_bot!(index, async |bot| {
      bot.release_use_item();
    });

    setmst(index, StateName::Looking, false).await;
    setmst(index, StateName::Interacting, false).await;
    setst(index, State::CanAttacking, true).await;
    setst(index, State::CanEating, true).await;
    setst(index, State::CanDrinking, true).await;
  }
}

impl SalarixiPlugin<AutoShieldOptions> for AutoShieldPlugin {
  fn new() -> Self {
    Self
  }

  fn activate(&'static self, index: u8, options: Arc<AutoShieldOptions>) {
    let task = tokio::spawn(async move {
      loop {
        self.tick(&index, &options).await;
        sleep!(options.tick_interval as u64);
      }
    });

    PLUGINS.push_task(&index, PluginName::AutoShield, task);
  }
}
