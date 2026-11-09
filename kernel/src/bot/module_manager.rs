use std::sync::{Arc, LazyLock};

use bytes::Bytes;
use hashbrown::HashMap;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::modules::*;
use crate::bot::systems::profile::{Profile, PROFILE_SYSTEM};
use crate::bot::traits::{ModuleOptions, SalarixiModule};
use crate::launch::process::current_options;
use crate::server::transfer::{emit_log, emit_msg, LogClass};
use crate::webhook::send_webhook;

pub static MODULES: LazyLock<ModuleManager> = LazyLock::new(|| ModuleManager::new());

#[derive(Debug, Index)]
enum ModuleName {
  Chat = 0,
  Action = 1,
  Inventory = 2,
  Movement = 3,
  AntiAfk = 4,
  Stalker = 5,
  Flight = 6,
  Killaura = 7,
  Scaffold = 8,
  AntiFall = 9,
  BowAim = 10,
  Stealer = 11,
  Miner = 12,
  Farmer = 13,
}

impl ModuleName {
  pub fn to_string(&self) -> String {
    match self {
      Self::Chat => "chat",
      Self::Action => "action",
      Self::Inventory => "inventory",
      Self::Movement => "movement",
      Self::AntiAfk => "anti-afk",
      Self::Stalker => "stalker",
      Self::Flight => "flight",
      Self::Killaura => "killaura",
      Self::Scaffold => "scaffold",
      Self::AntiFall => "anti-fall",
      Self::BowAim => "bow-aim",
      Self::Stealer => "stealer",
      Self::Miner => "miner",
      Self::Farmer => "farmer",
    }
    .to_string()
  }
}

#[derive(Debug)]
struct ControlMessage {
  module_name: ModuleName,
  group: String,
  raw_options: Bytes,
}

impl ControlMessage {
  fn from_bytes(mut buf: Bytes) -> Option<Self> {
    let module_index = u8::read(&mut buf)?;
    let module_name = ModuleName::from_index(module_index)?;
    let group = String::read(&mut buf)?;

    Some(Self {
      module_name,
      group,
      raw_options: buf,
    })
  }
}

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

  pub async fn control(&self, bytes: Bytes) -> Option<()> {
    let mut message = ControlMessage::from_bytes(bytes)?;

    let profiles = PROFILE_SYSTEM.get_all_connected().await;
    let total_count = profiles.len();

    let success_count = match message.module_name {
      ModuleName::Chat => self.switch_module(&self.chat, profiles, &mut message).await,
      ModuleName::Action => self.switch_module(&self.action, profiles, &mut message).await,
      ModuleName::Inventory => self.switch_module(&self.inventory, profiles, &mut message).await,
      ModuleName::Movement => self.switch_module(&self.movement, profiles, &mut message).await,
      ModuleName::AntiAfk => self.switch_module(&self.anti_afk, profiles, &mut message).await,
      ModuleName::Stalker => self.switch_module(&self.stalker, profiles, &mut message).await,
      ModuleName::Flight => self.switch_module(&self.flight, profiles, &mut message).await,
      ModuleName::Killaura => self.switch_module(&self.killaura, profiles, &mut message).await,
      ModuleName::Scaffold => self.switch_module(&self.scaffold, profiles, &mut message).await,
      ModuleName::AntiFall => self.switch_module(&self.anti_fall, profiles, &mut message).await,
      ModuleName::BowAim => self.switch_module(&self.bow_aim, profiles, &mut message).await,
      ModuleName::Stealer => self.switch_module(&self.stealer, profiles, &mut message).await,
      ModuleName::Miner => self.switch_module(&self.miner, profiles, &mut message).await,
      ModuleName::Farmer => self.switch_module(&self.farmer, profiles, &mut message).await,
    };

    let module_name = message.module_name.to_string();

    if let Some(opts) = current_options().await {
      if opts.basic.use_webhook && opts.webhook.send_actions {
        send_webhook(
          opts.webhook.url,
          format!(
            "Nhóm bot \"{}\" đã nhận lệnh \"{}\". Đã thực hiện {} trên {} bot.",
            message.group, module_name, success_count, total_count,
          ),
        );
      }
    }

    emit_log(
      format!(
        "Nhóm bot \"{}\" đã nhận lệnh \"{}\". Đã thực hiện {} trên {} bot",
        message.group, module_name, success_count, total_count,
      ),
      LogClass::Info,
    );

    emit_msg(
      "Quản lý",
      format!(
        "Nhóm bot \"{}\" đã nhận lệnh \"{}\". Đã thực hiện {} trên {} bot.",
        message.group, module_name, success_count, total_count
      ),
    );

    Some(())
  }

  async fn switch_module<M, O>(&self, module: &M, profiles: HashMap<u8, Profile>, message: &mut ControlMessage) -> u8
  where
    M: SalarixiModule<O>,
    O: ModuleOptions,
  {
    let options = Arc::new(match O::from_bytes(&mut message.raw_options) {
      Some(o) => o,
      None => return 0,
    });

    let mut success_count = 0;

    for (index, profile) in profiles {
      if profile.group != message.group {
        continue;
      }

      if module.switch(index, options.clone()).await {
        success_count += 1;
      }
    }

    success_count
  }
}
