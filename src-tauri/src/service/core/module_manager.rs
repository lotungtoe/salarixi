use once_cell::sync::Lazy;
use std::sync::Arc;

use crate::service::core::bot::push_task;
use crate::service::core::bot::PROFILES;
use crate::service::core::modules::*;

pub static MODULE_MANAGER: Lazy<Arc<ModuleManager>> = Lazy::new(|| Arc::new(ModuleManager::new()));

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

  pub async fn control(&'static self, name: String, options: serde_json::Value, group: String) {
    for username in PROFILES.get_all().into_keys() {
      let current_options = options.clone();

      if let Some(profile) = PROFILES.get(&username) {
        if profile.group != group {
          continue;
        }
      }

      match name.as_str() {
        "chat" => {
          let options: ChatOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          let mode = options.clone().mode;

          if mode.as_str() == "spamming" {
            self.chat.stop(&username);
          }

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.chat.enable(&nickname, &options).await;
            });

            if mode.as_str() == "spamming" {
              push_task(&username, "spamming", task);
            }
          } else if mode.as_str() == "spamming" {
            self.chat.stop(&username);
          }
        }
        "action" => {
          let options: ActionOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          let action = options.action.clone();

          self.action.stop(&username, action.as_str()).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.action.enable(&nickname, &options).await;
            });

            push_task(&username, action.as_str(), task);
          } else {
            self.action.stop(&username, action.as_str()).await;
          }
        }
        "inventory" => {
          let options: InventoryOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.inventory.interact(&username, &options).await;
        }
        "movement" => {
          let options: MovementOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.movement.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.movement.enable(&nickname, &options).await;
            });

            push_task(&username, "movement", task);
          } else {
            self.movement.stop(&username).await;
          }
        }
        "anti-afk" => {
          let options: AntiAfkOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.anti_afk.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.anti_afk.enable(&nickname, &options).await;
            });

            push_task(&username, "anti-afk", task);
          } else {
            self.anti_afk.stop(&username).await;
          }
        }
        "stalker" => {
          let options: StalkerOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.stalker.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.stalker.enable(&nickname, &options).await;
            });

            push_task(&username, "stalker", task);
          } else {
            self.stalker.stop(&username);
          }
        }
        "flight" => {
          let options: FlightOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.flight.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.flight.enable(&nickname, &options).await;
            });

            push_task(&username, "flight", task);
          } else {
            self.flight.stop(&username);
          }
        }
        "killaura" => {
          let options: KillauraOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.killaura.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.killaura.enable(&nickname, &options).await;
            });

            push_task(&username, "killaura", task);
          } else {
            self.killaura.stop(&username).await;
          }
        }
        "scaffold" => {
          let options: ScaffoldOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.scaffold.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.scaffold.enable(&nickname, &options).await;
            });

            push_task(&username, "scaffold", task);
          } else {
            self.scaffold.stop(&username).await;
          }
        }
        "anti-fall" => {
          let options: AntiFallOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.anti_fall.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.anti_fall.enable(&nickname, &options).await;
            });

            push_task(&username, "anti-fall", task);
          } else {
            self.anti_fall.stop(&username);
          }
        }
        "bow-aim" => {
          let options: BowAimOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.bow_aim.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.bow_aim.enable(&nickname, &options).await;
            });

            push_task(&username, "bow-aim", task);
          } else {
            self.bow_aim.stop(&username).await;
          }
        }
        "stealer" => {
          let options: StealerOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.stealer.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.stealer.enable(&nickname, &options).await;
            });

            push_task(&username, "stealer", task);
          } else {
            self.stealer.stop(&username);
          }
        }
        "miner" => {
          let options: MinerOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.miner.stop(&username).await;

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.miner.enable(&nickname, &options).await;
            });

            push_task(&username, "miner", task);
          } else {
            self.miner.stop(&username).await;
          }
        }
        "farmer" => {
          let options: FarmerOptions = serde_json::from_value(current_options)
            .map_err(|e| format!("Ошибка парсинга опций: {}", e))
            .unwrap();

          self.farmer.stop(&username);

          if options.state {
            let nickname = username.clone();

            let task = tokio::spawn(async move {
              self.farmer.enable(&nickname, &options).await;
            });

            push_task(&username, "farmer", task);
          } else {
            self.farmer.stop(&username);
          }
        }
        _ => {}
      }
    }
  }
}
