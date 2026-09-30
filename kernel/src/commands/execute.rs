use std::sync::atomic::Ordering;
use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use bytes::{BufMut, Bytes, BytesMut};
use salarixi_extensions::buffer::BufferExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use crate::bot::common::convert_inventory_slot_to_hotbar_slot;
use crate::bot::common_actions::{disconnect_bot, reset_bot};
use crate::bot::extensions::{BotInventoryExt, ClickMode};
use crate::bot::quick::run_quick_task;
use crate::bot::script::SCRIPT_EXECUTOR;
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::bot::utils::radar::RADAR;
use crate::bot::MODULES;
use crate::commands::enumeration::ClientCommand;
use crate::launch::options::{CaptchaSolveMode, CaptchaType, LaunchOptions};
use crate::launch::process::{current_options, process_is_active, process_status_payload, PROCESS_ACTIVITY};
use crate::launch::runner::{launch_bots_on_server, stop_bots_and_destroy_data};
use crate::server::{write_to_socket, write_to_sockets, Sockets};
use crate::{take_bot, take_profile};

async fn create_sync_buf() -> Option<Bytes> {
  let process = process_status_payload().await;

  let Some(opts) = current_options().await else {
    return None;
  };

  let mut buf = BytesMut::new();
  buf.put_u8(0x0C);
  buf.put_u8(if PROCESS_ACTIVITY.load(Ordering::SeqCst) {
    0x01
  } else {
    0x02
  });
  process.status_id.write(&mut buf);
  process.connected_bots.write(&mut buf);
  process.total_bots.write(&mut buf);
  buf.put_u8(if opts.basic.use_anti_captcha { 0x01 } else { 0x00 });
  buf.put_u8(match opts.captcha_bypass.captcha_type {
    CaptchaType::Web => 0x00,
    CaptchaType::Map => 0x01,
  });
  buf.put_u8(match opts.captcha_bypass.solve_mode {
    CaptchaSolveMode::Manual => 0x00,
    CaptchaSolveMode::Auto => 0x01,
  });
  opts.basic.enable_monitoring.write(&mut buf);
  opts.basic.chat_monitoring.write(&mut buf);
  opts.basic.inventory_monitoring.write(&mut buf);

  if PROCESS_ACTIVITY.load(Ordering::SeqCst) {
    let map = INDEX_SYSTEM.map.read().await;
    buf.put_u8(map.len() as u8);

    for username in map.iter() {
      username.write(&mut buf);
    }
  }

  Some(buf.freeze())
}

async fn synchronize(addr: &str, writer: &Arc<RwLock<OwnedWriteHalf>>, cipher: &Arc<Aes256Gcm>) {
  let Some(buf) = create_sync_buf().await else {
    return;
  };

  write_to_socket(addr, writer, cipher, &buf).await;
}

async fn synchronize_all(sockets: &Sockets) {
  let Some(buf) = create_sync_buf().await else {
    return;
  };

  write_to_sockets(sockets, &buf).await;
}

async fn session_chat(addr: String, data: &Bytes, sockets: &Sockets) {
  let mut buf = BytesMut::new();
  buf.put_u8(0x02);
  addr.write(&mut buf);
  buf.put_slice(data);

  write_to_sockets(sockets, &buf).await;
}

async fn remove_captcha(data: &Bytes, sockets: &Sockets) {
  let mut buf = BytesMut::new();
  buf.put_u8(0x07);
  buf.put_slice(data);

  write_to_sockets(sockets, &buf).await;
}

async fn launch_bots(data: &mut Bytes, sockets: &Sockets) {
  let options = match LaunchOptions::read(data) {
    Some(o) => o,
    None => {
      let mut buf = BytesMut::new();
      buf.put_u8(0x09);
      buf.put_u8(0x00);

      write_to_sockets(sockets, &buf).await;
      synchronize_all(sockets).await;

      return;
    }
  };

  let status = launch_bots_on_server(options).await;

  let mut buf = BytesMut::new();
  buf.put_u8(0x09);
  buf.put_u8(status);

  write_to_sockets(sockets, &buf).await;
  synchronize_all(sockets).await;
}

async fn stop_bots(sockets: &Sockets) {
  let status = stop_bots_and_destroy_data(true).await;

  let mut buf = BytesMut::new();
  buf.put_u8(0x0A);
  buf.put_u8(status);

  write_to_sockets(sockets, &buf).await;
  synchronize_all(sockets).await;
}

async fn common_action(data: &mut Bytes) -> Option<()> {
  let action = u8::read(data)?;
  let index = u8::read(data)?;

  match action {
    0x00 => {
      let message = String::read(data)?;
      take_bot!(&index, async |bot| bot.chat(message));
    }
    0x01 => {
      reset_bot(&index, false).await;
    }
    0x02 => {
      disconnect_bot(&index, false).await;
    }
    _ => {}
  }

  Some(())
}

async fn set_group(data: &mut Bytes, sockets: &Sockets) -> Option<()> {
  let index = u8::read(data)?;
  let group = String::read(data)?;

  take_profile!(&index, async |profile| {
    profile.group = group.clone();
  });

  let mut buf = BytesMut::new();
  buf.put_u8(0x0E);
  index.write(&mut buf);
  group.write(&mut buf);

  write_to_sockets(sockets, &buf).await;

  Some(())
}

async fn change_module_state(data: Bytes) {
  if process_is_active() {
    MODULES.control(data).await;
  }
}

async fn find_radar_target(
  data: &mut Bytes,
  addr: String,
  writer: &Arc<RwLock<OwnedWriteHalf>>,
  cipher: &Arc<Aes256Gcm>,
) -> Option<()> {
  let target = String::read(data)?;

  if let Some(info) = RADAR.find_target(target.clone()).await {
    let mut buf = BytesMut::new();
    buf.put_u8(0x0B);
    target.write(&mut buf);
    info.uuid.write(&mut buf);
    info.tx.write(&mut buf);
    info.ty.write(&mut buf);
    info.tz.write(&mut buf);
    info.ox.write(&mut buf);
    info.oz.write(&mut buf);

    write_to_socket(&addr, writer, cipher, &buf).await;
  }

  Some(())
}

async fn save_radar_data(data: &mut Bytes) -> Option<()> {
  let target = String::read(data)?;
  let path = String::read(data)?;
  let filename = String::read(data)?;
  let x = f64::read(data)?;
  let y = f64::read(data)?;
  let z = f64::read(data)?;

  RADAR.save_data(target, path, filename, x, y, z);

  Some(())
}

async fn follow_radar_target(data: &mut Bytes) -> Option<()> {
  let x = i32::read(data)?;
  let z = i32::read(data)?;

  RADAR.follow(x, z).await;

  Some(())
}

async fn quick_task(data: &mut Bytes) -> Option<()> {
  let id = u8::read(data)?;

  run_quick_task(id).await;

  Some(())
}

async fn execute_script(data: &mut Bytes) -> Option<()> {
  let script = String::read(data)?;
  let separately = bool::read(data)?;

  if !process_is_active() {
    return Some(());
  }

  if separately {
    for (index, _) in PROFILE_SYSTEM.get_all_connected().await {
      let Some(username) = INDEX_SYSTEM.username_by_index(&index).await else {
        continue;
      };

      let script_clone = script.clone();

      tokio::spawn(async move {
        SCRIPT_EXECUTOR.execute(username, index, script_clone);
      });
    }
  } else {
    SCRIPT_EXECUTOR.execute(String::new(), 0, script);
  }

  Some(())
}

async fn stop_script() {
  SCRIPT_EXECUTOR.stop();
}

async fn inventory_action(data: &mut Bytes) -> Option<()> {
  let index = u8::read(data)?;
  let slot = u8::read(data)?;
  let action = u8::read(data)?;

  let mut result = Some(());

  take_bot!(&index, async |bot| {
    match action {
      0x00 => {
        let Some(hotbar_slot) = convert_inventory_slot_to_hotbar_slot(slot as usize) else {
          return;
        };

        bot.set_selected_hotbar_slot(hotbar_slot);
      }
      0x01 => {
        bot.inventory_click(&index, slot as usize, ClickMode::Drop, true).await;
      }
      0x02 => {
        bot
          .inventory_click(&index, slot as usize, ClickMode::DropAll, true)
          .await;
      }
      0x03 => {
        bot.inventory_click(&index, slot as usize, ClickMode::Shift, true).await;
      }
      _ => result = None,
    }
  });

  result
}

async fn update_inventory(
  addr: &str,
  writer: &Arc<RwLock<OwnedWriteHalf>>,
  cipher: &Arc<Aes256Gcm>,
  data: &mut Bytes,
) -> Option<()> {
  let Some(opts) = current_options().await else {
    return Some(());
  };

  if !opts.basic.enable_monitoring || !opts.basic.inventory_monitoring {
    return Some(());
  }

  let index = u8::read(data)?;

  take_bot!(&index, async |bot| {
    let Some(menu) = bot.get_inventory_menu() else {
      return;
    };

    if menu.len() != 46 {
      return;
    }

    let mut buf = BytesMut::new();
    buf.put_u8(0x0F);
    index.write(&mut buf);
    (menu.slots().len() as u8).write(&mut buf);

    for (slot, item) in menu.slots().iter().enumerate() {
      (slot as u8).write(&mut buf);
      (if item.count() > 64 { 65 } else { item.count() as u8 }).write(&mut buf);

      if item.is_empty() {
        None::<String>.write(&mut buf);
      } else {
        Some(item.kind().to_string()[10..].to_string()).write(&mut buf);
      }
    }

    write_to_socket(addr, writer, cipher, &buf).await;
  });

  Some(())
}

async fn clear_inventory(data: &mut Bytes) -> Option<()> {
  let index = u8::read(data)?;

  take_bot!(&index, async |bot| {
    if let Some(menu) = bot.get_inventory_menu() {
      for (slot, _) in menu.slots().iter().enumerate() {
        bot.inventory_click(&index, slot, ClickMode::DropAll, false).await;
      }
    }
  });

  Some(())
}

/// Функция выполнения команды клиента
pub async fn execute_client_command(
  addr: String,
  writer: &Arc<RwLock<OwnedWriteHalf>>,
  cipher: &Arc<Aes256Gcm>,
  command: &ClientCommand,
  mut data: Bytes,
  sockets: &Sockets,
) -> Option<()> {
  println!(
    "[info :: {}] command \"{:?}\" received | length={}",
    addr,
    command,
    data.len()
  );

  match command {
    ClientCommand::SessionChat => session_chat(addr, &data, sockets).await,
    ClientCommand::RemoveCaptcha => remove_captcha(&data, sockets).await,
    ClientCommand::LaunchBots => launch_bots(&mut data, sockets).await,
    ClientCommand::StopBots => stop_bots(sockets).await,
    ClientCommand::QuickAction => common_action(&mut data).await?,
    ClientCommand::SetGroup => set_group(&mut data, sockets).await?,
    ClientCommand::ChangeModuleState => change_module_state(data).await,
    ClientCommand::FindRadarTarget => find_radar_target(&mut data, addr, writer, cipher).await?,
    ClientCommand::SaveRadarData => save_radar_data(&mut data).await?,
    ClientCommand::FollowRadarTarget => follow_radar_target(&mut data).await?,
    ClientCommand::QuickTask => quick_task(&mut data).await?,
    ClientCommand::ExecuteScript => execute_script(&mut data).await?,
    ClientCommand::StopScript => stop_script().await,
    ClientCommand::Synchronize => synchronize(&addr, writer, cipher).await,
    ClientCommand::InventoryAction => inventory_action(&mut data).await?,
    ClientCommand::UpdateInventory => update_inventory(&addr, writer, cipher, &mut data).await?,
    ClientCommand::ClearInventory => clear_inventory(&mut data).await?,
  }

  Some(())
}
