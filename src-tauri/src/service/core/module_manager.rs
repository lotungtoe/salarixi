use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use std::sync::Arc;

use crate::api::emit::emit_log;
use crate::service::core::bot::push_task;
use crate::service::core::bot::TaskName;
use crate::service::core::bot::PROFILES;
use crate::service::core::modules::*;

pub static MODULE_MANAGER: Lazy<Arc<ModuleManager>> = Lazy::new(|| Arc::new(ModuleManager::new()));

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
  /// Метод создания нового менеджера модулей
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
  pub async fn control(&'static self, name: String, options: serde_json::Value, group: String) -> Option<()> {
    for username in PROFILES.get_all().into_keys() {
      let current_options = options.clone();

      if let Some(profile) = PROFILES.get(&username) {
        if profile.group != group {
          continue;
        }
      }

      match name.as_str() {
        "chat" => {
          let options: ChatOptions = try_parse_options(current_options)?;

          let mode_clone = options.mode.clone();
          let mode = mode_clone.as_str();

          if mode == "spamming" {
            self.chat.stop(&username);
          }

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.chat.enable(&nickname, &options).await;
            });

            if mode == "spamming" {
              push_task(&username, TaskName::Spamming, task);
            }
          } else if mode == "spamming" {
            self.chat.stop(&username);
          }
        }
        "action" => {
          let options: ActionOptions = try_parse_options(current_options)?;

          let action = options.action.clone();

          self.action.stop(&username, action.as_str()).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.action.enable(&nickname, &options).await;
            });

            let task_name = match action.as_str() {
              "jumping" => TaskName::Jumping,
              "shifting" => TaskName::Shifting,
              "waving" => TaskName::Waving,
              _ => continue,
            };

            push_task(&username, task_name, task);
          } else {
            self.action.stop(&username, action.as_str()).await;
          }
        }
        "inventory" => {
          let options: InventoryOptions = try_parse_options(current_options)?;

          self.inventory.interact(&username, &options).await;
        }
        "movement" => {
          let options: MovementOptions = try_parse_options(current_options)?;

          self.movement.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.movement.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::Movement, task);
          } else {
            self.movement.stop(&username).await;
          }
        }
        "anti-afk" => {
          let options: AntiAfkOptions = try_parse_options(current_options)?;

          self.anti_afk.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.anti_afk.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::AntiAfk, task);
          } else {
            self.anti_afk.stop(&username).await;
          }
        }
        "stalker" => {
          let options: StalkerOptions = try_parse_options(current_options)?;

          self.stalker.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.stalker.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::Stalker, task);
          } else {
            self.stalker.stop(&username);
          }
        }
        "flight" => {
          let options: FlightOptions = try_parse_options(current_options)?;

          self.flight.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.flight.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::Flight, task);
          } else {
            self.flight.stop(&username);
          }
        }
        "killaura" => {
          let options: KillauraOptions = try_parse_options(current_options)?;

          self.killaura.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.killaura.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::Killaura, task);
          } else {
            self.killaura.stop(&username).await;
          }
        }
        "scaffold" => {
          let options: ScaffoldOptions = try_parse_options(current_options)?;

          self.scaffold.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.scaffold.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::Scaffold, task);
          } else {
            self.scaffold.stop(&username).await;
          }
        }
        "anti-fall" => {
          let options: AntiFallOptions = try_parse_options(current_options)?;

          self.anti_fall.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.anti_fall.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::AntiFall, task);
          } else {
            self.anti_fall.stop(&username);
          }
        }
        "bow-aim" => {
          let options: BowAimOptions = try_parse_options(current_options)?;

          self.bow_aim.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.bow_aim.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::BowAim, task);
          } else {
            self.bow_aim.stop(&username).await;
          }
        }
        "stealer" => {
          let options: StealerOptions = try_parse_options(current_options)?;

          self.stealer.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.stealer.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::Stealer, task);
          } else {
            self.stealer.stop(&username);
          }
        }
        "miner" => {
          let options: MinerOptions = try_parse_options(current_options)?;

          self.miner.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.miner.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::Miner, task);
          } else {
            self.miner.stop(&username).await;
          }
        }
        "farmer" => {
          let options: FarmerOptions = try_parse_options(current_options)?;

          self.farmer.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.farmer.enable(&nickname, &options).await;
            });

            push_task(&username, TaskName::Farmer, task);
          } else {
            self.farmer.stop(&username);
          }
        }
        _ => {}
      }
    }

    Some(())
  }
}
