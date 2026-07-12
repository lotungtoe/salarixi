use azalea::Client;
use serde::Deserialize;

use crate::bot::extensions::{BotInventoryExt, ClickMode};
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::traits::SalarixiModule;
use crate::emit::*;
use crate::take_bot;

#[derive(Deserialize)]
pub struct InventoryOptions {
  slot: Option<usize>,
  target_slot: Option<usize>,
  state: u8,
}

pub struct InventoryModule;

impl InventoryModule {
  pub fn new() -> Self {
    Self
  }

  pub async fn interact(bot: &Client, index: &u8, options: &InventoryOptions) {
    let Some(slot) = options.slot else {
      return;
    };

    match options.state {
      0 => {
        if slot <= 8 {
          bot.set_selected_hotbar_slot(slot as u8);
        } else {
          if let Some(username) = INDEX_SYSTEM.username_by_index(index).await {
            emit_log(
              format!("Бот {} не смог взять слот {} (неверный индекс слота)", username, slot),
              "error",
            );
          }
        }
      }
      1 => {
        bot.inventory_click(index, slot, ClickMode::DropAll, true).await;
      }
      2 => {
        bot.inventory_click(index, slot, ClickMode::Left, true).await;
      }
      3 => {
        bot.inventory_click(index, slot, ClickMode::Right, true).await;
      }
      4 => {
        bot
          .inventory_swap_click(index, slot, if let Some(t) = options.target_slot { t } else { 0 }, true)
          .await;
      }
      _ => {}
    }
  }
}

impl SalarixiModule<InventoryOptions> for InventoryModule {
  fn new() -> Self {
    Self
  }

  async fn switch(&self, index: u8, options: InventoryOptions) -> bool {
    tokio::spawn(async move {
      take_bot!(&index, async |bot| Self::interact(bot, &index, &options).await);
    });

    true
  }
}
