use hashbrown::HashMap;
use serde::Deserialize;

/// Структура опций запуска
#[derive(Debug, Deserialize, Clone)]
pub struct LaunchOptions {
  pub basic: BasicOptions,
  pub accounts: HashMap<String, AccountOptions>,
  pub plugins: PluginOptions,
  pub captcha_bypass: CaptchaBypassOptions,
  pub webhook: WebhookOptions,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BasicOptions {
  pub address: String,
  pub bots_count: i32,
  pub join_delay: u64,
  pub nickname_type: String,
  pub password_type: String,
  pub email_type: String,
  pub nickname_template: String,
  pub password_template: String,
  pub register_mode: String,
  pub register_command: String,
  pub register_template: String,
  pub register_min_delay: u64,
  pub register_max_delay: u64,
  pub register_trigger: String,
  pub login_mode: String,
  pub login_command: String,
  pub login_template: String,
  pub login_min_delay: u64,
  pub login_max_delay: u64,
  pub login_trigger: String,
  pub rejoin_delay: u64,
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
  pub use_anti_captcha: bool,
  pub use_webhook: bool,
  pub use_accounts: bool,
  pub proxy_list: Option<String>,
  pub script: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AccountOptions {
  pub password: Option<String>,
  pub email: Option<String>,
  pub proxy: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PluginOptions {
  pub auto_armor: bool,
  pub auto_totem: bool,
  pub auto_eat: bool,
  pub auto_potion: bool,
  pub auto_look: bool,
  pub auto_shield: bool,
  pub auto_repair: bool,
  pub auto_pearl: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CaptchaBypassOptions {
  pub captcha_type: String,
  pub captcha_subtype: String,
  pub solve_mode: String,
  pub regex: String,
  pub required_url_part: Option<String>,
  pub number_of_columns: u32,
  pub number_of_rows: u32,
  pub api_key: Option<String>,
  #[allow(unused)]
  pub api_service: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookOptions {
  pub url: Option<String>,
  pub send_information: bool,
  pub send_data: bool,
  pub send_actions: bool,
}
