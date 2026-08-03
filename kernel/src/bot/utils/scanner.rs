use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::emit::{emit_log, emit_msg, EmitEvent, EMIT_MANAGER};
use crate::result::CommandResult;
use crate::{failed, success};

static SCANNER_ROOT_TASK: Lazy<RwLock<Option<JoinHandle<()>>>> = Lazy::new(|| RwLock::new(None));
static SCANNER_EXTRA_TASKS: Lazy<RwLock<Vec<JoinHandle<()>>>> = Lazy::new(|| RwLock::new(Vec::new()));

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMeta {
  address: String,
  icon: Option<String>,
  online_players: i32,
  max_players: i32,
  version: String,
}

/// Функция парсинга диапазона IP-адресов
fn parse_ip_range(range: &str) -> CommandResult<Vec<String>> {
  let splitted = range.split('/').collect::<Vec<&str>>();

  let Ok(ip) = splitted[0].parse::<Ipv4Addr>() else {
    failed!("null address parsing error");
  };

  let Ok(designation) = splitted[1].parse::<u32>() else {
    failed!("designation parsing error");
  };

  if designation < 20 {
    failed!("number after '/' cannot be less than 20 or greater than 32");
  }

  let mask = if designation == 0 {
    0
  } else {
    !0u32 << (32 - designation)
  };

  let ip_num: u32 = ip.into();
  let null = ip_num & mask;
  let broadcast = null | !mask;

  let null_addr = Ipv4Addr::from(null);
  let null_octets = null_addr.as_octets();
  let broadcast_addr = Ipv4Addr::from(broadcast);
  let broadcast_octets = broadcast_addr.as_octets();

  let mut hosts = Vec::new();

  for a in 0..=255 {
    if a < null_octets[0] || a > broadcast_octets[0] {
      continue;
    }

    for b in 0..=255 {
      if b < null_octets[1] || b > broadcast_octets[1] {
        continue;
      }

      for c in 0..=255 {
        if c < null_octets[2] || c > broadcast_octets[2] {
          continue;
        }

        for d in 0..=255 {
          if d < null_octets[3] || d > broadcast_octets[3] {
            continue;
          }

          let octets = [a, b, c, d];
          if octets == *null_octets || octets == *broadcast_octets {
            continue;
          }

          let host = Ipv4Addr::from(octets).to_string();
          hosts.push(host);
        }
      }
    }
  }

  success!(hosts);
}

/// Функция сканирования сети на наличие Minecraft серверов
pub async fn start_scanning(range: String, mut task_count: usize, target_port: u32, timeout: u64) -> CommandResult<()> {
  let mut extra_tasks = SCANNER_EXTRA_TASKS.write().await;
  extra_tasks.iter().for_each(|t| t.abort());
  extra_tasks.clear();
  drop(extra_tasks);

  if let Some(task) = SCANNER_ROOT_TASK.read().await.as_ref() {
    task.abort();
  }

  let result = parse_ip_range(&range);
  let Some(data) = result.data else {
    failed!("{}", result.error.unwrap_or("unknown error".to_string()));
  };

  let total_hosts = data.len();

  EMIT_MANAGER.emit(EmitEvent::SetScannerStatus(1));
  EMIT_MANAGER.emit(EmitEvent::ScannerHostsScanned(0, total_hosts));
  EMIT_MANAGER.emit(EmitEvent::ScannerServersFound(0));

  if task_count > total_hosts {
    task_count = total_hosts;
  }

  let scanner_task = tokio::spawn(async move {
    let hosts = Arc::new(RwLock::new(data));
    let servers = Arc::new(AtomicUsize::new(0));
    let status_reset = Arc::new(AtomicBool::new(false));

    for _ in 0..task_count {
      let hosts_clone = hosts.clone();
      let servers_clone = servers.clone();
      let status_reset_clone = status_reset.clone();
      let task = tokio::spawn(async move {
        loop {
          let mut hosts_guard = hosts_clone.write().await;
          if hosts_guard.len() < 1 {
            if !status_reset_clone.load(Ordering::SeqCst) {
              status_reset_clone.store(true, Ordering::SeqCst);
              stop_scanning().await;
              emit_log("Сканирование серверов в указанном диапазоне завершено", "system");
              emit_msg("Сканер", "Сканирование серверов завершено");
            }

            break;
          }

          let Some(target_host) = hosts_guard.pop() else {
            continue;
          };

          drop(hosts_guard);

          let target_addr = format!("{}:{}", target_host, target_port);
          let result = tokio::time::timeout(
            Duration::from_millis(timeout),
            azalea::ping::ping_server(target_addr.clone()),
          )
          .await;

          if let Ok(Ok(data)) = result {
            let meta = ServerMeta {
              address: target_addr,
              icon: data.favicon,
              online_players: data.players.online,
              max_players: data.players.max,
              version: data.version.name,
            };

            EMIT_MANAGER.emit(EmitEvent::PushScannedServer(meta));

            let previous = servers_clone.fetch_add(1, Ordering::SeqCst);
            EMIT_MANAGER.emit(EmitEvent::ScannerServersFound(previous + 1));
          }

          EMIT_MANAGER.emit(EmitEvent::ScannerHostsScanned(
            total_hosts - hosts_clone.read().await.len(),
            total_hosts,
          ));
        }
      });

      SCANNER_EXTRA_TASKS.write().await.push(task);
    }
  });

  *SCANNER_ROOT_TASK.write().await = Some(scanner_task);

  success!(());
}

/// Функция остановки сканирования серверов
pub async fn stop_scanning() {
  let mut extra_tasks = SCANNER_EXTRA_TASKS.write().await;
  extra_tasks.iter().for_each(|t| t.abort());
  extra_tasks.clear();
  drop(extra_tasks);

  if let Some(task) = SCANNER_ROOT_TASK.read().await.as_ref() {
    task.abort();
  }

  *SCANNER_ROOT_TASK.write().await = None;

  EMIT_MANAGER.emit(EmitEvent::SetScannerStatus(0));
}
