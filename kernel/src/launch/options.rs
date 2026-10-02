use std::sync::Arc;

use hashbrown::HashMap;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use salarixi_macros::Index;

use crate::bot::extensions::EntityFilter;

/// Вспомогательная функция парсинга строкового списка сущностей
fn parse_entity_array(buf: &mut bytes::Bytes) -> Option<Vec<EntityFilter>> {
  let full_string = String::read(buf)?;
  let splitted = full_string.split(',').collect::<Vec<&str>>();
  let mut entities = Vec::new();

  for part in splitted {
    if let Some(filter) = EntityFilter::from_str(part.trim()) {
      entities.push(filter);
    }
  }

  Some(entities)
}

#[derive(Clone)]
pub struct LaunchOptions {
  pub basic: BasicOptions,
  pub accounts: HashMap<String, AccountOptions>,
  pub plugins: PluginOptions,
  pub captcha_bypass: CaptchaBypassOptions,
  pub webhook: WebhookOptions,
}

impl LaunchOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    let basic = BasicOptions::read(buf)?;

    let account_count = u8::read(buf)?;
    let mut accounts = HashMap::with_capacity(account_count as usize);

    for _ in 0..account_count {
      let username = String::read(buf)?;
      let options = AccountOptions::read(buf)?;
      accounts.insert(username, options);
    }

    let plugins = PluginOptions::read(buf)?;
    let captcha_bypass = CaptchaBypassOptions::read(buf)?;
    let webhook = WebhookOptions::read(buf)?;

    Some(Self {
      basic,
      accounts,
      plugins,
      captcha_bypass,
      webhook,
    })
  }
}

#[derive(Clone)]
pub struct BasicOptions {
  pub address: String,
  pub bots_count: u8,
  pub join_delay: u32,
  pub nickname_type: NicknameType,
  pub password_type: PasswordType,
  pub email_type: EmailType,
  pub nickname_template: String,
  pub password_template: String,
  pub register_mode: AuthMode,
  pub register_command: String,
  pub register_template: String,
  pub register_min_delay: u32,
  pub register_max_delay: u32,
  pub register_trigger: String,
  pub login_mode: AuthMode,
  pub login_command: String,
  pub login_template: String,
  pub login_min_delay: u32,
  pub login_max_delay: u32,
  pub login_trigger: String,
  pub rejoin_delay: u32,
  pub monitoring_update_rate: u32,
  pub view_distance: u8,
  pub humanoid_arm: Option<String>,
  pub use_auto_rejoin: bool,
  pub use_auto_register: bool,
  pub use_double_auth: bool,
  pub use_auto_login: bool,
  pub use_auto_respawn: bool,
  pub use_accept_rp: bool,
  pub use_pathfinder: bool,
  pub use_auto_script: bool,
  pub use_proxy: bool,
  pub use_accounts: bool,
  pub use_anti_captcha: bool,
  pub use_webhook: bool,
  pub enable_monitoring: bool,
  pub monitoring_optimization: bool,
  pub chat_monitoring: bool,
  pub inventory_monitoring: bool,
  pub proxy_list: Option<String>,
  pub script: Option<String>,
}

impl BasicOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      address: String::read(buf)?,
      bots_count: u8::read(buf)?,
      join_delay: u32::read(buf)?,
      nickname_type: NicknameType::from_index(u8::read(buf)?)?,
      password_type: PasswordType::from_index(u8::read(buf)?)?,
      email_type: EmailType::from_index(u8::read(buf)?)?,
      nickname_template: String::read(buf)?,
      password_template: String::read(buf)?,
      register_mode: AuthMode::from_index(u8::read(buf)?)?,
      register_command: String::read(buf)?,
      register_template: String::read(buf)?,
      register_min_delay: u32::read(buf)?,
      register_max_delay: u32::read(buf)?,
      register_trigger: String::read(buf)?,
      login_mode: AuthMode::from_index(u8::read(buf)?)?,
      login_command: String::read(buf)?,
      login_template: String::read(buf)?,
      login_min_delay: u32::read(buf)?,
      login_max_delay: u32::read(buf)?,
      login_trigger: String::read(buf)?,
      rejoin_delay: u32::read(buf)?,
      monitoring_update_rate: u32::read(buf)?,
      view_distance: u8::read(buf)?,
      humanoid_arm: Option::read(buf)?,
      use_auto_rejoin: bool::read(buf)?,
      use_auto_register: bool::read(buf)?,
      use_double_auth: bool::read(buf)?,
      use_auto_login: bool::read(buf)?,
      use_auto_respawn: bool::read(buf)?,
      use_accept_rp: bool::read(buf)?,
      use_pathfinder: bool::read(buf)?,
      use_auto_script: bool::read(buf)?,
      use_proxy: bool::read(buf)?,
      use_accounts: bool::read(buf)?,
      use_anti_captcha: bool::read(buf)?,
      use_webhook: bool::read(buf)?,
      enable_monitoring: bool::read(buf)?,
      monitoring_optimization: bool::read(buf)?,
      chat_monitoring: bool::read(buf)?,
      inventory_monitoring: bool::read(buf)?,
      proxy_list: Option::read(buf)?,
      script: Option::read(buf)?,
    })
  }
}

#[derive(Clone, Index)]
pub enum NicknameType {
  Legit = 0x00,
  Random = 0x01,
  Custom = 0x02,
}

impl NicknameType {
  pub fn to_str(&self) -> &str {
    match self {
      Self::Legit => "legit",
      Self::Random => "random",
      Self::Custom => "custom",
    }
  }
}

#[derive(Clone, Index)]
pub enum PasswordType {
  Without = 0x00,
  Legit = 0x01,
  Random = 0x02,
  Custom = 0x03,
}

impl PasswordType {
  pub fn to_str(&self) -> &str {
    match self {
      Self::Without => "without",
      Self::Legit => "legit",
      Self::Random => "random",
      Self::Custom => "custom",
    }
  }
}

#[derive(Clone, Index)]
pub enum EmailType {
  Without = 0x00,
  Random = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum AuthMode {
  Default = 0x00,
  Trigger = 0x01,
}

#[derive(Clone)]
pub struct AccountOptions {
  pub initial_group: Option<String>,
  pub password: Option<String>,
  pub email: Option<String>,
  pub proxy: Option<String>,
  pub registered: bool,
  pub logined: bool,
}

impl AccountOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      initial_group: Option::read(buf)?,
      password: Option::read(buf)?,
      email: Option::read(buf)?,
      proxy: Option::read(buf)?,
      registered: bool::read(buf)?,
      logined: bool::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct PluginOptions {
  pub instant_armor_equip: Arc<InstantArmorEquipOptions>,
  pub auto_totem: Arc<AutoTotemOptions>,
  pub auto_eat: Arc<AutoEatOptions>,
  pub potion_consumer: Arc<PotionConsumerOptions>,
  pub auto_look: Arc<AutoLookOptions>,
  pub auto_shield: Arc<AutoShieldOptions>,
  pub auto_mending: Arc<AutoMendingOptions>,
  pub pearl_leave: Arc<PearlLeaveOptions>,
}

impl PluginOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      instant_armor_equip: Arc::new(InstantArmorEquipOptions::read(buf)?),
      auto_totem: Arc::new(AutoTotemOptions::read(buf)?),
      auto_eat: Arc::new(AutoEatOptions::read(buf)?),
      potion_consumer: Arc::new(PotionConsumerOptions::read(buf)?),
      auto_look: Arc::new(AutoLookOptions::read(buf)?),
      auto_shield: Arc::new(AutoShieldOptions::read(buf)?),
      auto_mending: Arc::new(AutoMendingOptions::read(buf)?),
      pearl_leave: Arc::new(PearlLeaveOptions::read(buf)?),
    })
  }
}

#[derive(Clone)]
pub struct InstantArmorEquipOptions {
  pub enabled: bool,
  pub tick_interval: u32,
  pub delay_after_no_armor: u32,
  pub include_helmet: bool,
  pub include_chestplate: bool,
  pub include_leggings: bool,
  pub include_boots: bool,
  pub compare_by_material: bool,
  pub compare_by_enchantments: bool,
  pub compare_by_durability: bool,
  pub adaptation_by_dimension: bool,
  pub adaptation_by_physical_state: bool,
}

impl InstantArmorEquipOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      enabled: bool::read(buf)?,
      tick_interval: u32::read(buf)?,
      delay_after_no_armor: u32::read(buf)?,
      include_helmet: bool::read(buf)?,
      include_chestplate: bool::read(buf)?,
      include_leggings: bool::read(buf)?,
      include_boots: bool::read(buf)?,
      compare_by_material: bool::read(buf)?,
      compare_by_enchantments: bool::read(buf)?,
      compare_by_durability: bool::read(buf)?,
      adaptation_by_dimension: bool::read(buf)?,
      adaptation_by_physical_state: bool::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct AutoTotemOptions {
  pub enabled: bool,
  pub tick_interval: u32,
  pub delay_after_taking: u32,
  pub ignore_item_in_offhand: bool,
  pub ignore_states: bool,
}

impl AutoTotemOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      enabled: bool::read(buf)?,
      tick_interval: u32::read(buf)?,
      delay_after_taking: u32::read(buf)?,
      ignore_item_in_offhand: bool::read(buf)?,
      ignore_states: bool::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct AutoEatOptions {
  pub enabled: bool,
  pub tick_interval: u32,
  pub minimum_satiety_threshold: u32,
  pub minimum_health_threshold: f32,
  pub delay_after_no_eat: u32,
  pub effects_check: bool,
  pub reduce_eat_chance_on_combat: bool,
}

impl AutoEatOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      enabled: bool::read(buf)?,
      tick_interval: u32::read(buf)?,
      minimum_satiety_threshold: u32::read(buf)?,
      minimum_health_threshold: f32::read(buf)?,
      delay_after_no_eat: u32::read(buf)?,
      effects_check: bool::read(buf)?,
      reduce_eat_chance_on_combat: bool::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct PotionConsumerOptions {
  pub enabled: bool,
  pub tick_interval: u32,
  pub minimum_health_threshold: f32,
  pub include_drinking_potion: bool,
  pub include_splash_potion: bool,
  pub delay_after_no_potion: u32,
  pub reduce_drink_chance_on_combat: bool,
}

impl PotionConsumerOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      enabled: bool::read(buf)?,
      tick_interval: u32::read(buf)?,
      minimum_health_threshold: f32::read(buf)?,
      include_drinking_potion: bool::read(buf)?,
      include_splash_potion: bool::read(buf)?,
      delay_after_no_potion: u32::read(buf)?,
      reduce_drink_chance_on_combat: bool::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct AutoLookOptions {
  pub enabled: bool,
  pub tick_interval: u32,
  pub include_entities: Vec<EntityFilter>,
  pub distance: f64,
  pub with_eye_height: bool,
}

impl AutoLookOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      enabled: bool::read(buf)?,
      tick_interval: u32::read(buf)?,
      include_entities: parse_entity_array(buf)?,
      distance: f64::read(buf)?,
      with_eye_height: bool::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct AutoShieldOptions {
  pub enabled: bool,
  pub tick_interval: u32,
  pub delay_after_not_equipped: u32,
  pub dangerous_entities: Vec<EntityFilter>,
  pub distance: f64,
  pub ignore_item_in_offhand: bool,
}

impl AutoShieldOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      enabled: bool::read(buf)?,
      tick_interval: u32::read(buf)?,
      delay_after_not_equipped: u32::read(buf)?,
      dangerous_entities: parse_entity_array(buf)?,
      distance: f64::read(buf)?,
      ignore_item_in_offhand: bool::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct AutoMendingOptions {
  pub enabled: bool,
  pub tick_interval: u32,
  pub percent_of_unbroken_item_durability: f32,
}

impl AutoMendingOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      enabled: bool::read(buf)?,
      tick_interval: u32::read(buf)?,
      percent_of_unbroken_item_durability: f32::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct PearlLeaveOptions {
  pub enabled: bool,
  pub tick_interval: u32,
  pub minimum_health_threshold: f32,
  pub maximum_health_threshold: f32,
  pub delay_after_no_pearl: u32,
}

impl PearlLeaveOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      enabled: bool::read(buf)?,
      tick_interval: u32::read(buf)?,
      minimum_health_threshold: f32::read(buf)?,
      maximum_health_threshold: f32::read(buf)?,
      delay_after_no_pearl: u32::read(buf)?,
    })
  }
}

#[derive(Clone)]
pub struct CaptchaBypassOptions {
  pub captcha_type: CaptchaType,
  pub captcha_subtype: CaptchaSubtype,
  pub solve_mode: CaptchaSolveMode,
  pub captcha_size: CaptchaSize,
  pub regex: String,
  pub required_url_part: Option<String>,
  pub number_of_columns: u32,
  pub number_of_rows: u32,
  pub max_pause: u32,
  pub user_id: Option<String>,
  pub api_key: Option<String>,
  pub api_service: CaptchaApiService,
}

impl CaptchaBypassOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      captcha_type: CaptchaType::from_index(u8::read(buf)?)?,
      captcha_subtype: CaptchaSubtype::from_index(u8::read(buf)?)?,
      solve_mode: CaptchaSolveMode::from_index(u8::read(buf)?)?,
      captcha_size: CaptchaSize::from_index(u8::read(buf)?)?,
      regex: String::read(buf)?,
      required_url_part: Option::read(buf)?,
      number_of_columns: u32::read(buf)?,
      number_of_rows: u32::read(buf)?,
      max_pause: u32::read(buf)?,
      user_id: Option::read(buf)?,
      api_key: Option::read(buf)?,
      api_service: CaptchaApiService::from_index(u8::read(buf)?)?,
    })
  }
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaType {
  Web = 0x00,
  Map = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaSubtype {
  Inventory = 0x00,
  Frame = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaSolveMode {
  Manual = 0x00,
  Auto = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaSize {
  Fixed = 0x00,
  Dynamic = 0x01,
}

#[derive(Clone, PartialEq, Index)]
pub enum CaptchaApiService {
  TwoCaptcha = 0x00,
  TrueCaptcha = 0x01,
}

#[derive(Clone)]
pub struct WebhookOptions {
  pub url: Option<String>,
  pub send_information: bool,
  pub send_data: bool,
  pub send_actions: bool,
}

impl WebhookOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      url: Option::read(buf)?,
      send_information: bool::read(buf)?,
      send_data: bool::read(buf)?,
      send_actions: bool::read(buf)?,
    })
  }
}
