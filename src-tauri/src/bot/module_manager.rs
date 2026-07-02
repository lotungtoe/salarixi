use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use std::sync::Arc;

use crate::api::emit::{emit_log, emit_msg};
use crate::bot::modules::*;
use crate::bot::systems::PROFILE_SYSTEM;
use crate::bot::traits::SalarixiModule;
use crate::launch::process::current_options;
use crate::other::webhook::send_webhook;

pub static MODULES: Lazy<Arc<ModuleManager>> = Lazy::new(|| Arc::new(ModuleManager::new()));

/// Вспомогательный метод парсинга опций
fn try_parse_options<T>(value: serde_json::Value) -> Option<T>
where
  T: DeserializeOwned,
{
  match serde_json::from_value(value) {
    Ok(o) => Some(o),
    Err(e) => {
      emit_log(format!("Ошибка парсинга указанных опций: {}", e), "error");
      None
    }
  }
}

/// Менеджер модулей.
/// Изначально он хранит в себе объекты всех существующих модулей.
/// Позволяет конвертировать JSON-данные в Rust структуры, распределять группы, включать / выключать определённые модули.
pub struct ModuleManager {
  chat: ChatModule,
  action: ActionModule,
  inventory: InventoryModule,
  movement: MovementModule,
  anti_afk: AntiAfkModule,
  stalker: StalkerModule,
  flight: FlightModule,
  killaura: KillauraModule,
  scaffold: ScaffoldModule,
  anti_fall: AntiFallModule,
  bow_aim: BowAimModule,
  stealer: StealerModule,
  miner: MinerModule,
  farmer: FarmerModule,
}

impl ModuleManager {
  /// Метод создания менеджера модулей
  pub fn new() -> Self {
    Self {
      chat: ChatModule::new(),
      action: ActionModule::new(),
      inventory: InventoryModule::new(),
      movement: MovementModule::new(),
      anti_afk: AntiAfkModule::new(),
      stalker: StalkerModule::new(),
      flight: FlightModule::new(),
      killaura: KillauraModule::new(),
      scaffold: ScaffoldModule::new(),
      anti_fall: AntiFallModule::new(),
      bow_aim: BowAimModule::new(),
      stealer: StealerModule::new(),
      miner: MinerModule::new(),
      farmer: FarmerModule::new(),
    }
  }

  /// Метод управления определённым менеджером
  pub async fn control(&self, module_name: String, options: serde_json::Value, group: String) -> Option<()> {
    let profiles = PROFILE_SYSTEM.get_all_connected().await;
    let total_count = profiles.len();
    let mut success_count = 0;

    for (username, _) in profiles {
      let cur_opts = options.clone();

      if let Some(profile) = PROFILE_SYSTEM.get(&username).await {
        if profile.group != group {
          continue;
        }
      }

      let success;

      match module_name.as_str() {
        "chat" => {
          let options: ChatOptions = try_parse_options(cur_opts)?;
          success = self.chat.switch(username, options).await;
        }
        "action" => {
          let options: ActionOptions = try_parse_options(cur_opts)?;
          success = self.action.switch(username, options).await;
        }
        "inventory" => {
          let options: InventoryOptions = try_parse_options(cur_opts)?;
          success = self.inventory.switch(username, options).await;
        }
        "movement" => {
          let options: MovementOptions = try_parse_options(cur_opts)?;
          success = self.movement.switch(username, options).await;
        }
        "anti-afk" => {
          let options: AntiAfkOptions = try_parse_options(cur_opts)?;
          success = self.anti_afk.switch(username, options).await;
        }
        "stalker" => {
          let options: StalkerOptions = try_parse_options(cur_opts)?;
          success = self.stalker.switch(username, options).await;
        }
        "flight" => {
          let options: FlightOptions = try_parse_options(cur_opts)?;
          success = self.flight.switch(username, options).await;
        }
        "killaura" => {
          let options: KillauraOptions = try_parse_options(cur_opts)?;
          success = self.killaura.switch(username, options).await;
        }
        "scaffold" => {
          let options: ScaffoldOptions = try_parse_options(cur_opts)?;
          success = self.scaffold.switch(username, options).await;
        }
        "anti-fall" => {
          let options: AntiFallOptions = try_parse_options(cur_opts)?;
          success = self.anti_fall.switch(username, options).await;
        }
        "bow-aim" => {
          let options: BowAimOptions = try_parse_options(cur_opts)?;
          success = self.bow_aim.switch(username, options).await;
        }
        "stealer" => {
          let options: StealerOptions = try_parse_options(cur_opts)?;
          success = self.stealer.switch(username, options).await;
        }
        "miner" => {
          let options: MinerOptions = try_parse_options(cur_opts)?;
          success = self.miner.switch(username, options).await;
        }
        "farmer" => {
          let options: FarmerOptions = try_parse_options(cur_opts)?;
          success = self.farmer.switch(username, options).await;
        }
        _ => continue,
      }

      if success {
        success_count += 1;
      }
    }

    if let Some(opts) = current_options().await {
      if opts.basic.use_webhook && opts.webhook.send_actions {
        send_webhook(
          opts.webhook.url,
          format!(
            "Группа ботов \"{}\" получила команду \"{}\". Выполнили {} из {} ботов.",
            group, module_name, success_count, total_count,
          ),
        );
      }
    }

    emit_log(
      format!(
        "Группа ботов \"{}\" получила команду \"{}\". Выполнили {} из {} ботов",
        group, module_name, success_count, total_count,
      ),
      "extended",
    );

    emit_msg(
      "Управление",
      format!(
        "Группа ботов \"{}\" получила команду \"{}\". Выполнили {} из {} ботов.",
        group, module_name, success_count, total_count
      ),
    );

    Some(())
  }
}
