use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::emit::{EmitEvent, EMIT_MANAGER};

static SCANNER_ROOT_TASK: Lazy<RwLock<Option<JoinHandle<()>>>> = Lazy::new(|| RwLock::new(None));
static SCANNER_EXTRA_TASKS: Lazy<RwLock<Vec<JoinHandle<()>>>> = Lazy::new(|| RwLock::new(Vec::new()));

#[derive(Clone, Serialize)]
pub struct ServerMeta {
  address: String,
  icon: Option<String>,
  version: String,
}

/// Функция парсинга диапазона IP-адресов
fn parse_ip_range(range: &str) -> Vec<String> {
  let splitted = range.split('/').collect::<Vec<&str>>();

  let Ok(ip) = splitted[0].parse::<Ipv4Addr>() else {
    return Vec::new();
  };

  let Ok(designation) = splitted[1].parse::<u32>() else {
    return Vec::new();
  };

  if designation < 20 {
    return Vec::new();
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

  hosts
}

/// Функция сканирования сети на наличие Minecraft серверов
pub async fn start_scanning(range: String, task_count: u8, target_port: u32, timeout: u64) {
  let mut extra_tasks = SCANNER_EXTRA_TASKS.write().await;
  extra_tasks.iter().for_each(|t| t.abort());
  extra_tasks.clear();
  drop(extra_tasks);

  if let Some(task) = SCANNER_ROOT_TASK.read().await.as_ref() {
    task.abort();
  }

  let parsed = parse_ip_range(&range);
  if parsed.is_empty() {
    return;
  }

  let total_hosts = parsed.len();

  EMIT_MANAGER.emit(EmitEvent::SetScannerStatus(1));
  EMIT_MANAGER.emit(EmitEvent::ScannerHostsScanned(0, total_hosts));
  EMIT_MANAGER.emit(EmitEvent::ScannerServersFound(0));

  let scanner_task = tokio::spawn(async move {
    let hosts = Arc::new(RwLock::new(parsed));
    let servers = Arc::new(AtomicUsize::new(0));

    for _ in 0..task_count {
      let hosts_clone = hosts.clone();
      let servers_clone = servers.clone();
      let task = tokio::spawn(async move {
        loop {
          let mut hosts_guard = hosts_clone.write().await;
          if hosts_guard.len() < 1 {
            break;
          }

          let target_host = hosts_guard.remove(0);
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

  EMIT_MANAGER.emit(EmitEvent::ScannerHostsScanned(0, 0));
  EMIT_MANAGER.emit(EmitEvent::ScannerServersFound(0));
  EMIT_MANAGER.emit(EmitEvent::SetScannerStatus(0));
}
