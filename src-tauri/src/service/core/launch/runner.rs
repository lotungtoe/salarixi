use azalea::app::PluginGroup;
use azalea::auto_reconnect::AutoReconnectDelay;
use azalea::prelude::*;
use azalea::protocol::connect::Proxy;
use azalea::swarm::*;
use azalea::JoinOpts;
use socks5_impl::protocol::UserKey;
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use crate::api::emit::{emit_log, emit_msg};
use crate::service::core::controllers::{ProfileProxy, PROFILES, STATES, TASKS};
use crate::service::core::handlers::single::single_handler;
use crate::service::core::handlers::swarm::swarm_handler;
use crate::service::core::plugin_manager::PLUGIN_MANAGER;
use crate::service::core::registry::{registry_event_loop, BOT_REGISTRY};
use crate::service::core::utils::MAP_ACCUMULATOR;
use crate::service::webhook::send_webhook;

use super::generators::{generate_email, generate_unique_username, generate_username_or_password};
use super::options::{AccountOptions, LaunchOptions};
use super::state::{active_bots_count, current_options, process_is_active, set_options, ACTIVE};

#[derive(Debug)]
struct CustomAccount {
  object: Account,
  options: Option<AccountOptions>,
}

fn clear_proxy(proxy: &str) -> String {
  proxy
    .trim_start_matches("socks5://")
    .trim_start_matches("socks4://")
    .trim_start_matches("https://")
    .trim_start_matches("http://")
    .to_string()
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

  emit_log("Подготовка...".to_string(), "extended");

  set_options(options.clone());

  ACTIVE.store(true, Ordering::Relaxed);

  tokio::spawn(registry_event_loop());

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
            accounts.push(CustomAccount {
              object: Account::offline(username),
              options: Some(opts.clone()),
            });

            PROFILES.register(username, opts.password.clone(), opts.email.clone());
          }
        } else {
          for _ in 0..options.basic.bots_count {
            let Some(username) =
              generate_unique_username(&options.basic.nickname_type, &options.basic.nickname_template)
            else {
              continue;
            };

            let password = generate_username_or_password(
              "password",
              &options.basic.password_type,
              &options.basic.password_template,
            );

            let email = generate_email(&options.basic.email_type);

            accounts.push(CustomAccount {
              object: Account::offline(&username),
              options: None,
            });

            PROFILES.register(&username, password, email);
          }
        }

        if options.basic.use_proxy || options.basic.use_accounts {
          let mut accounts_with_opts = Vec::new();

          if options.basic.use_accounts {
            for account in accounts.into_iter() {
              let mut join_opts = JoinOpts::new();

              if let Some(account_opts) = account.options.clone() {
                if let Some(proxy) = account_opts.proxy {
                  let mut profile_proxy = ProfileProxy {
                    ip_address: None,
                    proxy: None,
                    username: None,
                    password: None,
                  };

                  let clean_proxy = clear_proxy(&proxy);

                  profile_proxy.proxy = Some(clean_proxy.to_string());

                  if let Ok(addr) = clean_proxy.parse::<SocketAddr>() {
                    let mut proxy = Proxy::new(addr, None);

                    if let Some(username) = account_opts.proxy_username {
                      if let Some(password) = account_opts.proxy_password {
                        profile_proxy.username = Some(username.clone());
                        profile_proxy.password = Some(password.clone());

                        proxy.auth = Some(UserKey {
                          username: username,
                          password: password,
                        });
                      }
                    }

                    join_opts = join_opts.proxy(proxy);

                    let split_address: Vec<&str> = clean_proxy.split(":").collect();

                    let Some(ip_address) = split_address.get(0) else {
                      continue;
                    };

                    profile_proxy.ip_address = Some(ip_address.to_string());

                    PROFILES.set_proxy(&account.object.username, profile_proxy);
                  }
                };
              };

              accounts_with_opts.push((account, join_opts));
            }
          } else {
            for (i, account) in accounts.into_iter().enumerate() {
              let mut join_opts = JoinOpts::new();

              if let Some(proxy_list) = &options.basic.proxy_list {
                let list: Vec<&str> = proxy_list.split("\n").collect();

                if !list.is_empty() {
                  let mut profile_proxy = ProfileProxy {
                    ip_address: None,
                    proxy: None,
                    username: None,
                    password: None,
                  };

                  let proxy = list[i % list.len()];
                  let clean_proxy = clear_proxy(&proxy);

                  let proxy_address: Vec<&str> = clean_proxy.split("@").collect();

                  let Some(address) = proxy_address.get(1) else {
                    continue;
                  };

                  profile_proxy.proxy = Some(address.to_string());

                  if let Ok(addr) = address.parse::<SocketAddr>() {
                    let mut proxy = Proxy::new(addr, None);

                    if let Some(auth) = proxy_address.get(0) {
                      let split_auth: Vec<&str> = auth.split(":").collect();

                      let Some(username) = split_auth.get(0) else {
                        continue;
                      };

                      let Some(password) = split_auth.get(1) else {
                        continue;
                      };

                      profile_proxy.username = Some(username.to_string());
                      profile_proxy.password = Some(password.to_string());

                      proxy.auth = Some(UserKey {
                        username: username.to_string(),
                        password: password.to_string(),
                      });
                    }

                    join_opts = join_opts.proxy(proxy);

                    let split_address: Vec<&str> = address.split(":").collect();

                    let Some(ip_address) = split_address.get(0) else {
                      continue;
                    };

                    profile_proxy.ip_address = Some(ip_address.to_string());

                    PROFILES.set_proxy(&account.object.username, profile_proxy);
                  }
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

        emit_log("Подготовка окончена".to_string(), "extended");

        let _ = flow.start(options.basic.address).await;
      });

      local_set.await;
    });
  });

  true
}

/// Функция остановки ботов
pub async fn stop_bots_and_destroy_data() -> bool {
  emit_msg("Система", "Остановка ботов...".to_string());

  if !process_is_active() {
    emit_log(
      format!("Остановка ботов невозможна: The process is not active now"),
      "warning",
    );

    return false;
  }

  let active_bots = active_bots_count();

  PLUGIN_MANAGER.clear().await;
  TASKS.clear().await;

  tokio::time::sleep(Duration::from_secs(2)).await;

  STATES.clear().await;
  PROFILES.clear().await;
  MAP_ACCUMULATOR.clear_all();
  BOT_REGISTRY.clear();

  ACTIVE.store(false, Ordering::SeqCst);

  if let Some(options) = current_options() {
    if options.basic.use_webhook {
      send_webhook(
        options.webhook.url.clone(),
        format!("{} ботов было остановлено", active_bots),
      );
    }
  }

  emit_msg("Система", "Остановка ботов завершена");
  emit_log(format!("Остановка ботов завершена"), "info");

  true
}
