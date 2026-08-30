use std::io::{Error, ErrorKind};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use bytes::{BufMut, Bytes, BytesMut};
use salarixi_extensions::buffer::BufferExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use crate::bot::common_actions::{disconnect_bot, reset_bot};
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
  buf.put_u8(process.status_id);
  buf.put_u8(process.connected_bots);
  buf.put_u8(process.total_bots);
  buf.put_u8(if opts.basic.use_anti_captcha { 0x01 } else { 0x00 });
  buf.put_u8(match opts.captcha_bypass.captcha_type {
    CaptchaType::Web => 0x00,
    CaptchaType::Map => 0x01,
  });
  buf.put_u8(match opts.captcha_bypass.solve_mode {
    CaptchaSolveMode::Manual => 0x00,
    CaptchaSolveMode::Auto => 0x01,
  });

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

async fn common_action(data: &mut Bytes) -> std::io::Result<()> {
  let action = u8::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
  let index = u8::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

  match action {
    0x00 => {
      let message = String::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
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

  Ok(())
}

async fn set_group(data: &mut Bytes, sockets: &Sockets) -> std::io::Result<()> {
  let index = u8::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
  let group = String::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

  take_profile!(&index, async |profile| {
    profile.group = group.clone();
  });

  let mut buf = BytesMut::new();
  buf.put_u8(0x0E);
  index.write(&mut buf);
  group.write(&mut buf);

  write_to_sockets(sockets, &buf).await;

  Ok(())
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
) -> std::io::Result<()> {
  let target = String::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

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

  Ok(())
}

async fn save_radar_data(data: &mut Bytes) -> std::io::Result<()> {
  let target = String::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
  let path = String::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
  let filename = String::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
  let x = f64::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
  let y = f64::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
  let z = f64::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

  RADAR.save_data(target, path, filename, x, y, z);

  Ok(())
}

async fn follow_radar_target(data: &mut Bytes) -> std::io::Result<()> {
  let x = i32::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
  let z = i32::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

  RADAR.follow(x, z).await;

  Ok(())
}

async fn quick_task(data: &mut Bytes) -> std::io::Result<()> {
  let id = u8::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

  run_quick_task(id).await;

  Ok(())
}

async fn execute_script(data: &mut Bytes) -> std::io::Result<()> {
  let script = String::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
  let separately = bool::read(data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

  if !process_is_active() {
    return Ok(());
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

  Ok(())
}

async fn stop_script() {
  SCRIPT_EXECUTOR.stop();
}

/// Функция выполнения команды клиента
pub async fn execute_client_command(
  addr: String,
  writer: &Arc<RwLock<OwnedWriteHalf>>,
  cipher: &Arc<Aes256Gcm>,
  command: &ClientCommand,
  mut data: Bytes,
  sockets: &Sockets,
) -> std::io::Result<()> {
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
  }

  Ok(())
}
