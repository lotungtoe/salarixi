use std::io::{Error, ErrorKind};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use crate::bot::quick::QUICK_TASKS;
use crate::bot::quick_action::{disconnect_bot, reset_bot, send_message_from_bot, QuickAction};
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
use crate::take_profile;

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

  // TODO: Разбить все команды по отдельным функциям
  match command {
    ClientCommand::SessionChat => {
      let mut buf = BytesMut::new();
      buf.put_u8(0x02);
      addr.write(&mut buf);
      buf.put_slice(&data);

      write_to_sockets(sockets, &buf).await;
    }
    ClientCommand::RemoveCaptcha => {
      let mut buf = BytesMut::new();
      buf.put_u8(0x07);
      buf.put_slice(&data);

      write_to_sockets(sockets, &buf).await;
    }
    ClientCommand::LaunchBots => {
      let options = match LaunchOptions::read(&mut data) {
        Some(o) => o,
        None => {
          let mut buf = BytesMut::new();
          buf.put_u8(0x09);
          buf.put_u8(0x00);

          write_to_sockets(sockets, &buf).await;
          synchronize_all(sockets).await;

          return Ok(());
        }
      };

      let status = launch_bots_on_server(options).await;

      let mut buf = BytesMut::new();
      buf.put_u8(0x09);
      buf.put_u8(status);

      write_to_sockets(sockets, &buf).await;
      synchronize_all(sockets).await;
    }
    ClientCommand::StopBots => {
      let status = stop_bots_and_destroy_data(true).await;

      let mut buf = BytesMut::new();
      buf.put_u8(0x0A);
      buf.put_u8(status);

      write_to_sockets(sockets, &buf).await;
      synchronize_all(sockets).await;
    }
    ClientCommand::QuickAction => {
      let Some(action) = QuickAction::from_index(data.try_get_u8()?) else {
        return Ok(());
      };

      let username = String::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

      match action {
        QuickAction::SendMessage => {
          let message = String::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
          send_message_from_bot(username, message).await;
        }
        QuickAction::Reset => {
          reset_bot(username).await;
        }
        QuickAction::Disconnect => {
          disconnect_bot(username).await;
        }
      }
    }
    ClientCommand::SetGroup => {
      let username = String::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
      let group = String::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

      if let Some(index) = INDEX_SYSTEM.index_by_username(&username).await {
        take_profile!(&index, async |profile| {
          profile.group = group;
        });
      }
    }
    ClientCommand::ChangeModuleState => {
      if process_is_active() {
        MODULES.control(data).await;
      }
    }
    ClientCommand::RadarFindTarget => {
      let target = String::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

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
    }
    ClientCommand::SaveRadarData => {
      let target = String::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
      let path = String::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
      let filename = String::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
      let x = f64::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
      let y = f64::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
      let z = f64::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

      RADAR.save_data(target, path, filename, x, y, z);
    }
    ClientCommand::FollowRadarTarget => {
      let x = i32::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
      let z = i32::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

      RADAR.follow(x, z).await;
    }
    ClientCommand::QuickTask => {
      let id = u8::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

      QUICK_TASKS.execute(id).await;
    }
    ClientCommand::ExecuteScript => {
      let script = String::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;
      let separately = bool::read(&mut data).ok_or(Error::new(ErrorKind::InvalidData, "options parsing error"))?;

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
    }
    ClientCommand::StopScript => {
      SCRIPT_EXECUTOR.stop();
    }
    // TODO: Реализовать нормальную логику синхронизации клиентов
    ClientCommand::Synchronize => {
      synchronize(&addr, writer, cipher).await;
    }
  }

  Ok(())
}

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
