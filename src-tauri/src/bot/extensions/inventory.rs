use azalea::container::ContainerHandleRef;
use azalea::core::direction::Direction;
use azalea::core::position::BlockPos;
use azalea::entity::inventory::Inventory;
use azalea::inventory::components::ItemName;
use azalea::inventory::operations::ThrowClick;
use azalea::inventory::Menu;
use azalea::prelude::*;
use azalea::protocol::packets::game::s_player_action::Action;
use azalea::protocol::packets::game::ServerboundPlayerAction;
use azalea::registry::builtin::ItemKind;
use std::time::Duration;
use tokio::time::sleep;

use crate::bot::common::convert_inventory_slot_to_hotbar_slot;
use crate::bot::extensions::BotMovementExt;
use crate::bot::systems::{getst, setmst, setst, State, StateName};
use crate::other::tools::randnum;

pub trait BotInventoryExt {
  fn get_selected_slot(&self) -> u8;
  fn get_current_inventory(&self) -> Option<ContainerHandleRef>;
  fn get_inventory_menu(&self) -> Option<Menu>;
  fn find_empty_slot_in_invenotry(&self) -> Option<usize>;
  fn start_interacting_with_inventory(&self) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn stop_interacting_with_inventory(&self) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn move_item_to_offhand(&self, kind: ItemKind) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn inventory_click(
    &self,
    slot: usize,
    mode: ClickMode,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn inventory_click_on(
    &self,
    name: &str,
    mode: ClickMode,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn take_item(&self, source_slot: usize, lock: bool) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn inventory_swap_click(
    &self,
    source_slot: usize,
    target_slot: usize,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn inventory_move_item(
    &self,
    kind: ItemKind,
    source_slot: usize,
    target_slot: usize,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
}

/// Режим клика в инвентаре
pub enum ClickMode {
  Left,
  Right,
  Shift,
  Drop,
  DropAll,
}

impl From<&str> for ClickMode {
  fn from(value: &str) -> Self {
    match value {
      "left" => Self::Left,
      "right" => Self::Right,
      "drop" => Self::Drop,
      "drop-all" => Self::DropAll,
      _ => Self::Shift,
    }
  }
}

impl BotInventoryExt for Client {
  fn get_current_inventory(&self) -> Option<ContainerHandleRef> {
    if let Some(inventory) = self.get_component::<Inventory>() {
      return Some(ContainerHandleRef::new(inventory.id, self.clone()));
    }

    None
  }

  fn get_selected_slot(&self) -> u8 {
    if let Some(inventory) = self.get_component::<Inventory>() {
      return inventory.selected_hotbar_slot;
    }

    0
  }

  fn get_inventory_menu(&self) -> Option<Menu> {
    if let Some(inventory) = self.get_component::<Inventory>() {
      return Some(inventory.menu().clone());
    }

    None
  }

  fn find_empty_slot_in_invenotry(&self) -> Option<usize> {
    let Some(menu) = self.get_inventory_menu() else {
      return None;
    };

    for (slot, item) in menu.slots().iter().enumerate() {
      if slot > 9 {
        if item.is_empty() {
          return Some(slot);
        }
      }
    }

    None
  }

  async fn start_interacting_with_inventory(&self) {
    let username = self.username();

    self.freeze_move().await;

    setst(&username, State::CanEating, false).await;
    setst(&username, State::CanDrinking, false).await;
    setst(&username, State::CanAttacking, false).await;

    setmst(&username, StateName::Interacting, true).await;
  }

  async fn stop_interacting_with_inventory(&self) {
    let username = self.username();

    self.unfreeze_move().await;

    setst(&username, State::CanEating, true).await;
    setst(&username, State::CanDrinking, true).await;
    setst(&username, State::CanAttacking, true).await;

    setmst(&username, StateName::Interacting, false).await;
  }

  async fn take_item(&self, source_slot: usize, lock: bool) {
    let username = self.username();

    if lock {
      if !getst(&username, State::CanInteracting).await {
        return;
      }

      self.start_interacting_with_inventory().await;
    }

    if let Some(hotbar_slot) = convert_inventory_slot_to_hotbar_slot(source_slot) {
      if self.get_selected_slot() != hotbar_slot {
        self.set_selected_hotbar_slot(hotbar_slot);
      }
    } else {
      if let Some(empty_slot) = self.find_empty_slot_in_invenotry() {
        self.inventory_swap_click(source_slot, empty_slot as usize, false).await;

        if let Some(slot) = convert_inventory_slot_to_hotbar_slot(empty_slot as usize) {
          if self.get_selected_slot() != slot {
            sleep(Duration::from_millis(50)).await;
            self.set_selected_hotbar_slot(slot);
          }
        }
      } else {
        let random_slot = randnum(36, 44) as usize;

        self.inventory_click(random_slot, ClickMode::Shift, false).await;
        sleep(Duration::from_millis(50)).await;
        self.inventory_swap_click(source_slot, random_slot, false).await;

        sleep(Duration::from_millis(50)).await;

        let hotbar_slot = convert_inventory_slot_to_hotbar_slot(random_slot).unwrap_or(0);

        if self.get_selected_slot() != hotbar_slot {
          self.set_selected_hotbar_slot(hotbar_slot);
        }
      }
    }

    if lock {
      self.stop_interacting_with_inventory().await;
    }
  }

  async fn move_item_to_offhand(&self, kind: ItemKind) {
    if let Some(menu) = self.get_inventory_menu() {
      if let Some(item) = menu.slot(45) {
        if item.kind() == kind {
          return;
        }
      }
    }

    self.write_packet(ServerboundPlayerAction {
      action: Action::SwapItemWithOffhand,
      pos: BlockPos::new(0, 0, 0),
      direction: Direction::Down,
      seq: 0,
    });
  }

  async fn inventory_swap_click(&self, source_slot: usize, target_slot: usize, lock: bool) {
    let username = self.username();

    if let Some(inventory) = self.get_current_inventory() {
      if lock {
        if !getst(&username, State::CanInteracting).await {
          return;
        }

        self.start_interacting_with_inventory().await;
      }

      if let Some(menu) = self.get_inventory_menu() {
        if let Some(item) = menu.slot(target_slot) {
          if !item.is_empty() {
            if let Some(empty_slot) = self.find_empty_slot_in_invenotry() {
              inventory.left_click(target_slot);
              sleep(Duration::from_millis(50)).await;
              inventory.left_click(empty_slot);
            } else {
              self.inventory_click(target_slot, ClickMode::DropAll, false).await;
            }

            sleep(Duration::from_millis(50)).await;
          }
        }
      }

      inventory.left_click(source_slot);
      sleep(Duration::from_millis(50)).await;
      inventory.left_click(target_slot);

      sleep(Duration::from_millis(50)).await;

      if lock {
        self.stop_interacting_with_inventory().await;
      }
    }
  }

  async fn inventory_click(&self, slot: usize, mode: ClickMode, lock: bool) {
    let username = self.username();

    if let Some(inventory) = self.get_current_inventory() {
      if lock {
        if !getst(&username, State::CanInteracting).await {
          return;
        }

        self.start_interacting_with_inventory().await;
      }

      match mode {
        ClickMode::Left => {
          inventory.left_click(slot);
        }
        ClickMode::Right => {
          inventory.right_click(slot);
        }
        ClickMode::Shift => {
          inventory.shift_click(slot);
        }
        ClickMode::Drop => {
          inventory.click(ThrowClick::Single { slot: slot as u16 });
        }
        ClickMode::DropAll => {
          inventory.click(ThrowClick::All { slot: slot as u16 });
        }
      }

      if lock {
        self.stop_interacting_with_inventory().await;
      }
    }
  }

  async fn inventory_click_on(&self, name: &str, mode: ClickMode, lock: bool) {
    let string_name = name.to_string();

    if let Some(menu) = self.get_inventory_menu() {
      for (slot, item) in menu.slots().iter().enumerate() {
        let Some(item_name) = item.get_component::<ItemName>() else {
          continue;
        };

        if item_name.name.to_string() == string_name {
          self.inventory_click(slot, mode, lock).await;
          break;
        }
      }
    }
  }

  async fn inventory_move_item(&self, kind: ItemKind, source_slot: usize, target_slot: usize, lock: bool) {
    if let Some(menu) = self.get_inventory_menu() {
      if let Some(item) = menu.slot(target_slot) {
        if item.kind() == kind {
          return;
        }
      }
    }

    self.inventory_swap_click(source_slot, target_slot, lock).await;
  }
}
