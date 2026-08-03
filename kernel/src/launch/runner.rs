use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use azalea::app::PluginGroup;
use azalea::auto_reconnect::AutoReconnectDelay;
use azalea::prelude::*;
use azalea::protocol::connect::Proxy;
use azalea::swarm::*;
use azalea::JoinOpts;
use socks5_impl::protocol::UserKey;

use crate::bot::handlers::single::single_handler;
use crate::bot::handlers::swarm::swarm_handler;
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::{ProfileProxy, PROFILE_SYSTEM};
use crate::bot::systems::registry::REGISTRY_SYSTEM;
use crate::bot::systems::states::STATE_SYSTEM;
use crate::bot::systems::tasks::TASK_SYSTEM;
use crate::bot::utils::acaptcha::MAP_ACCUMULATOR;
use crate::bot::PLUGINS;
use crate::emit::{emit_log, emit_msg};
use crate::other::webhook::send_webhook;
use crate::sleep;

use super::generators::{generate_email, generate_unique_username, generate_username_or_password};
use super::options::{AccountOptions, LaunchOptions};
use super::process::{current_options, process_is_active, set_options, set_process_activity};

/// Структура кастомного аккаунта бота
#[derive(Debug)]
struct CustomAccount {
  object: Account,
  options: AccountOptions,
  index: u8,
}

/// Вспомогательная функция очистки прокси
fn clear_proxy(proxy: &str) -> &str {
  proxy
    .trim_start_matches("socks5://")
    .trim_start_matches("socks4://")
    .trim_start_matches("https://")
    .trim_start_matches("http://")
}

/// Функция парсинга и обработки прокси
async fn processs_proxy(index: &u8, proxy: &str, join_opts: JoinOpts) -> JoinOpts {
  let mut profile_proxy = ProfileProxy {
    ip_address: None,
    proxy: None,
    username: None,
    password: None,
  };

  let clean_proxy = clear_proxy(proxy);
  let proxy_parts: Vec<&str> = clean_proxy.split("@").collect();

  if let Some(str_addr) = proxy_parts.get(1) {
    if let Ok(socket_addr) = str_addr.parse::<SocketAddr>() {
      profile_proxy.proxy = Some(str_addr.to_string());

      let mut proxy = Proxy::new(socket_addr, None);

      if let Some(auth) = proxy_parts.get(0) {
        let splited_auth: Vec<&str> = auth.split(":").collect();

        let Some(username) = splited_auth.get(0) else {
          return join_opts;
        };

        let Some(password) = splited_auth.get(1) else {
          return join_opts;
        };

        profile_proxy.username = Some(username.to_string());
        profile_proxy.password = Some(password.to_string());

        proxy.auth = Some(UserKey {
          username: username.to_string(),
          password: password.to_string(),
        });
      }

      let splited_addr: Vec<&str> = str_addr.split(":").collect();
      let Some(ip_addr) = splited_addr.get(0) else {
        return join_opts;
      };

      profile_proxy.ip_address = Some(ip_addr.to_string());
      PROFILE_SYSTEM.set_proxy(index, profile_proxy).await;

      return join_opts.proxy(proxy);
    }
  } else {
    if let Some(str_addr) = proxy_parts.get(0) {
      if let Ok(socket_addr) = str_addr.parse::<SocketAddr>() {
        profile_proxy.proxy = Some(str_addr.to_string());

        let proxy = Proxy::new(socket_addr, None);
        let splited_addr: Vec<&str> = str_addr.split(":").collect();
        let Some(ip_addr) = splited_addr.get(0) else {
          return join_opts;
        };

        profile_proxy.ip_address = Some(ip_addr.to_string());
        PROFILE_SYSTEM.set_proxy(index, profile_proxy).await;

        return join_opts.proxy(proxy);
      }
    }
  }

  join_opts
}

/// Функция запуска ботов на сервер
pub async fn launch_bots_on_server(options: LaunchOptions) -> bool {
  if process_is_active() {
    emit_log(
      format!("Запуск ботов невозможен: The process is already active"),
      "warning",
    );

    return false;
  }

  emit_log("Подготовка...", "extended");

  set_options(options.clone()).await;
  set_process_activity(true);

  if options.basic.use_webhook {
    send_webhook(
      options.webhook.url.clone(),
      format!("Запуск ботов на сервер {}...", options.basic.address),
    );
  }

  if options.basic.use_webhook && options.webhook.send_data {
    send_webhook(options.webhook.url.clone(), format!("Опции запуска: {:#?}", options));
  }

  thread::spawn(move || {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
      let local_set = tokio::task::LocalSet::new();

      let mut swarm_plugins = azalea::DefaultPlugins.build();

      if !options.basic.use_auto_rejoin {
        swarm_plugins = swarm_plugins.disable::<azalea::auto_reconnect::AutoReconnectPlugin>();
      } else {
        AutoReconnectDelay::new(Duration::from_millis(options.basic.rejoin_delay));
      }

      let mut bot_plugins = azalea::bot::DefaultBotPlugins.build();

      if !options.basic.use_auto_respawn {
        bot_plugins = bot_plugins.disable::<azalea::auto_respawn::AutoRespawnPlugin>();
      }

      if !options.basic.use_accept_rp {
        bot_plugins = bot_plugins.disable::<azalea::accept_resource_packs::AcceptResourcePacksPlugin>();
      }

      if !options.basic.use_pathfinder {
        bot_plugins = bot_plugins.disable::<azalea::pathfinder::PathfinderPlugin>();
      }

      local_set.spawn_local(async move {
        let mut flow = SwarmBuilder::new_without_plugins()
          .add_plugins(swarm_plugins)
          .add_plugins(bot_plugins)
          .add_plugins(azalea::swarm::DefaultSwarmPlugins)
          .join_delay(Duration::from_millis(options.basic.join_delay))
          .set_swarm_handler(swarm_handler)
          .set_handler(single_handler);

        let mut accounts = Vec::new();

        if options.basic.use_accounts {
          for (username, opts) in &options.accounts {
            let index = INDEX_SYSTEM.register(username).await;

            accounts.push(CustomAccount {
              object: Account::offline(username),
              options: opts.clone(),
              index,
            });

            PROFILE_SYSTEM
              .register(index, opts.password.clone(), opts.email.clone())
              .await;
          }
        } else {
          for _ in 0..options.basic.bots_count {
            let Some(username) =
              generate_unique_username(&options.basic.nickname_type, &options.basic.nickname_template).await
            else {
              continue;
            };

            let index = INDEX_SYSTEM.register(&username).await;

            let password = generate_username_or_password(
              "password",
              &options.basic.password_type,
              &options.basic.password_template,
            );

            let email = generate_email(&options.basic.email_type);

            accounts.push(CustomAccount {
              object: Account::offline(&username),
              options: AccountOptions {
                initial_group: None,
                password: None,
                email: None,
                proxy: None,
              },
              index,
            });

            PROFILE_SYSTEM.register(index, password, email).await;
          }
        }

        if options.basic.use_proxy || options.basic.use_accounts {
          let mut accounts_with_opts = Vec::new();

          if options.basic.use_accounts {
            for account in accounts.into_iter() {
              if let Some(initial_group) = &account.options.initial_group {
                PROFILE_SYSTEM.set_str(&account.index, "group", initial_group).await;
              }

              let mut join_opts = JoinOpts::new();

              if let Some(proxy) = &account.options.proxy {
                join_opts = processs_proxy(&account.index, proxy, join_opts).await;
              }

              accounts_with_opts.push((account, join_opts));
            }
          } else {
            for (i, account) in accounts.into_iter().enumerate() {
              let mut join_opts = JoinOpts::new();

              if let Some(proxy_list) = &options.basic.proxy_list {
                let list: Vec<&str> = proxy_list.split("\n").collect();

                if !list.is_empty() {
                  let proxy = list[i % list.len()];
                  join_opts = processs_proxy(&account.index, proxy, join_opts).await;
                }
              }

              accounts_with_opts.push((account, join_opts));
            }
          }

          for (account, opts) in accounts_with_opts {
            flow = flow.add_account_with_opts(account.object, opts);
          }
        } else {
          for account in accounts {
            flow = flow.add_account(account.object);
          }
        }

        emit_log("Подготовка окончена", "extended");

        let _ = flow.start(options.basic.address).await;
      });

      local_set.await;
    });
  });

  true
}

/// Функция остановки ботов
pub async fn stop_bots_and_destroy_data() -> bool {
  emit_msg("Система", "Остановка ботов...");

  if !process_is_active() {
    emit_log(
      format!("Остановка ботов невозможна: The process is not active now"),
      "warning",
    );

    return false;
  }

  let total_bots = PROFILE_SYSTEM.len().await;

  PLUGINS.clear().await;
  TASK_SYSTEM.clear().await;
  sleep!(1000);
  REGISTRY_SYSTEM.destroy_swarm().await;
  sleep!(1000);
  STATE_SYSTEM.clear().await;
  PROFILE_SYSTEM.clear().await;
  MAP_ACCUMULATOR.clear_all().await;
  REGISTRY_SYSTEM.clear().await;
  INDEX_SYSTEM.clear().await;

  set_process_activity(false);

  if let Some(options) = current_options().await {
    if options.basic.use_webhook {
      send_webhook(
        options.webhook.url.clone(),
        format!("{} ботов было остановлено", total_bots),
      );
    }
  }

  emit_msg("Система", "Остановка ботов завершена");
  emit_log("Остановка ботов завершена", "info");

  true
}
