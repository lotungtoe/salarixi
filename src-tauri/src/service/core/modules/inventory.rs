use azalea::Client;
use serde::Deserialize;

use crate::api::emit::*;
use crate::service::core::extensions::{BotInventoryExt, ClickMode};
use crate::service::core::registry::BOT_REGISTRY;
use crate::service::core::traits::SalarixiModule;

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

  pub async fn interact(bot: &Client, username: &str, options: &InventoryOptions) {
    if let Some(s) = options.slot {
      if options.state.as_str() == "select" {
        if s <= 8 {
          bot.set_selected_hotbar_slot(s as u8);
        } else {
          emit_log(
            format!("Бот {} не смог взять слот {} (неверный индекс слота)", username, s),
            "error",
          );
        }
      } else {
        match options.state.as_str() {
          "drop" => {
            bot.inventory_click(s, ClickMode::DropAll, true);
          }
          "left-click" => {
            bot.inventory_click(s, ClickMode::Left, true);
          }
          "right-click" => {
            bot.inventory_click(s, ClickMode::Right, true);
          }
          "swap" => {
            bot
              .inventory_swap_click(s, if let Some(t) = options.target_slot { t } else { 0 }, true)
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

  fn switch(&self, username: String, options: InventoryOptions) -> bool {
    tokio::spawn(async move {
      BOT_REGISTRY
        .async_get_bot(&username, async |bot| Self::interact(bot, &username, &options).await)
        .await;
    });

    true
  }
}
