use hashbrown::HashMap;
use salarixi_extensions::buffer::BufferExt;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct BasicOptions {
  pub address: String,
  pub bots_count: u8,
  pub join_delay: u32,
  pub nickname_type: String,
  pub password_type: String,
  pub email_type: String,
  pub nickname_template: String,
  pub password_template: String,
  pub register_mode: String,
  pub register_command: String,
  pub register_template: String,
  pub register_min_delay: u32,
  pub register_max_delay: u32,
  pub register_trigger: String,
  pub login_mode: String,
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
  pub use_anti_captcha: bool,
  pub use_webhook: bool,
  pub use_accounts: bool,
  pub proxy_list: Option<String>,
  pub script: Option<String>,
}

impl BasicOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      address: String::read(buf)?,
      bots_count: u8::read(buf)?,
      join_delay: u32::read(buf)?,
      nickname_type: String::read(buf)?,
      password_type: String::read(buf)?,
      email_type: String::read(buf)?,
      nickname_template: String::read(buf)?,
      password_template: String::read(buf)?,
      register_mode: String::read(buf)?,
      register_command: String::read(buf)?,
      register_template: String::read(buf)?,
      register_min_delay: u32::read(buf)?,
      register_max_delay: u32::read(buf)?,
      register_trigger: String::read(buf)?,
      login_mode: String::read(buf)?,
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
      use_anti_captcha: bool::read(buf)?,
      use_webhook: bool::read(buf)?,
      use_accounts: bool::read(buf)?,
      proxy_list: Option::read(buf)?,
      script: Option::read(buf)?,
    })
  }
}

#[derive(Debug, Clone)]
pub struct AccountOptions {
  pub initial_group: Option<String>,
  pub password: Option<String>,
  pub email: Option<String>,
  pub proxy: Option<String>,
}

impl AccountOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      initial_group: Option::read(buf)?,
      password: Option::read(buf)?,
      email: Option::read(buf)?,
      proxy: Option::read(buf)?,
    })
  }
}

#[derive(Debug, Clone)]
pub struct PluginOptions {
  pub instant_armor_equip: bool,
  pub auto_totem: bool,
  pub auto_eat: bool,
  pub potion_consumer: bool,
  pub auto_look: bool,
  pub auto_shield: bool,
  pub auto_mending: bool,
  pub pearl_leave: bool,
}

impl PluginOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      instant_armor_equip: bool::read(buf)?,
      auto_totem: bool::read(buf)?,
      auto_eat: bool::read(buf)?,
      potion_consumer: bool::read(buf)?,
      auto_look: bool::read(buf)?,
      auto_shield: bool::read(buf)?,
      auto_mending: bool::read(buf)?,
      pearl_leave: bool::read(buf)?,
    })
  }
}

#[derive(Debug, Clone)]
pub struct CaptchaBypassOptions {
  pub captcha_type: String,
  pub captcha_subtype: String,
  pub solve_mode: String,
  pub regex: String,
  pub required_url_part: Option<String>,
  pub number_of_columns: u32,
  pub number_of_rows: u32,
  pub api_key: Option<String>,
  pub api_service: String,
}

impl CaptchaBypassOptions {
  pub fn read(buf: &mut bytes::Bytes) -> Option<Self> {
    Some(Self {
      captcha_type: String::read(buf)?,
      captcha_subtype: String::read(buf)?,
      solve_mode: String::read(buf)?,
      regex: String::read(buf)?,
      required_url_part: Option::read(buf)?,
      number_of_columns: u32::read(buf)?,
      number_of_rows: u32::read(buf)?,
      api_key: Option::read(buf)?,
      api_service: String::read(buf)?,
    })
  }
}

#[derive(Debug, Clone)]
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
