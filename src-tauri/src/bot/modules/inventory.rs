use azalea::Client;
use serde::Deserialize;

use crate::bot::extensions::{BotInventoryExt, ClickMode};
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::traits::SalarixiModule;
use crate::emit::*;
use crate::take_bot;

pub struct InventoryModule;

#[derive(Deserialize)]
pub struct InventoryOptions {
  pub slot: Option<usize>,
  pub target_slot: Option<usize>,
  pub state: String,
}

impl InventoryModule {
  pub fn new() -> Self {
    Self
  }

  pub async fn interact(bot: &Client, index: &u8, options: &InventoryOptions) {
    if let Some(s) = options.slot {
      if options.state.as_str() == "select" {
        if s <= 8 {
          bot.set_selected_hotbar_slot(s as u8);
        } else {
          if let Some(username) = INDEX_SYSTEM.username_by_index(index).await {
            emit_log(
              format!("Бот {} не смог взять слот {} (неверный индекс слота)", username, s),
              "error",
            );
          }
        }
      } else {
        match options.state.as_str() {
          "drop" => {
            bot.inventory_click(index, s, ClickMode::DropAll, true).await;
          }
          "left-click" => {
            bot.inventory_click(index, s, ClickMode::Left, true).await;
          }
          "right-click" => {
            bot.inventory_click(index, s, ClickMode::Right, true).await;
          }
          "swap" => {
            bot
              .inventory_swap_click(index, s, if let Some(t) = options.target_slot { t } else { 0 }, true)
              .await;
          }
          _ => {}
        }
      }
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
