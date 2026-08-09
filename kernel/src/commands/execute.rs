use std::io::{Error, ErrorKind};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use bytes::{Buf, BufMut, BytesMut};
use hashbrown::HashMap;
use salarixi_extensions::buffer::BufferExt;
use salarixi_extensions::index::IndexExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;

use crate::bot::quick::QUICK_TASKS;
use crate::bot::quick_action::{disconnect_bot, reset_bot, send_message_from_bot, QuickAction};
use crate::bot::script::SCRIPT_EXECUTOR;
use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::bot::utils::radar::RADAR;
use crate::bot::MODULES;
use crate::commands::enumeration::ClientCommand;
use crate::launch::options::LaunchOptions;
use crate::launch::process::{process_is_active, process_status_payload, PROCESS_ACTIVITY};
use crate::launch::runner::{launch_bots_on_server, stop_bots_and_destroy_data};
use crate::server::write_bytes_to_sockets;
use crate::take_profile;

/// Функция выполнения команды клиента
pub async fn execute_client_command(
  addr: String,
  writer: &Arc<RwLock<OwnedWriteHalf>>,
  command: &ClientCommand,
  mut data: BytesMut,
  sockets: &Arc<RwLock<HashMap<String, (Arc<RwLock<OwnedReadHalf>>, Arc<RwLock<OwnedWriteHalf>>)>>>,
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
      addr.write(&mut buf);
      buf.put_slice(&data);

      let mut final_buf = BytesMut::new();
      final_buf.put_u8(0x04);
      final_buf.put_u32(buf.len() as u32);
      final_buf.put_slice(&buf);

      write_bytes_to_sockets(sockets, final_buf.freeze()).await;
    }
    ClientCommand::RemoveCaptcha => {
      let mut buf = BytesMut::new();
      buf.put_u8(0x08);
      buf.put_u32(data.len() as u32);
      buf.put_slice(&data);

      write_bytes_to_sockets(sockets, buf.freeze()).await;
    }
    ClientCommand::LaunchBots => {
      let options = match LaunchOptions::read(&mut data) {
        Some(o) => o,
        None => {
          let mut buf = BytesMut::new();
          buf.put_u8(0x09);
          buf.put_u32(1);
          buf.put_u8(0x00);

          write_bytes_to_sockets(sockets, buf.freeze()).await;

          return Ok(());
        }
      };

      let status = launch_bots_on_server(options).await;

      let mut buf = BytesMut::new();
      buf.put_u8(0x09);
      buf.put_u32(1);
      buf.put_u8(status);

      write_bytes_to_sockets(sockets, buf.freeze()).await;

      let process = process_status_payload().await;
      let mut buf = BytesMut::new();
      buf.put_u8(0x0E);
      buf.put_u32(4);
      buf.put_u8(if PROCESS_ACTIVITY.load(Ordering::SeqCst) {
        0x01
      } else {
        0x02
      });
      buf.put_u8(process.status_id);
      buf.put_u8(process.connected_bots);
      buf.put_u8(process.total_bots);

      write_bytes_to_sockets(sockets, buf.freeze()).await;
    }
    ClientCommand::StopBots => {
      let status = stop_bots_and_destroy_data().await;

      let mut buf = BytesMut::new();
      buf.put_u8(0x0A);
      buf.put_u32(1);
      buf.put_u8(status);

      write_bytes_to_sockets(sockets, buf.freeze()).await;

      let process = process_status_payload().await;
      let mut buf = BytesMut::new();
      buf.put_u8(0x0E);
      buf.put_u32(4);
      buf.put_u8(if PROCESS_ACTIVITY.load(Ordering::SeqCst) {
        0x01
      } else {
        0x02
      });
      buf.put_u8(process.status_id);
      buf.put_u8(process.connected_bots);
      buf.put_u8(process.total_bots);

      write_bytes_to_sockets(sockets, buf.freeze()).await;
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
        target.write(&mut buf);
        info.uuid.write(&mut buf);
        info.tx.write(&mut buf);
        info.ty.write(&mut buf);
        info.tz.write(&mut buf);
        info.ox.write(&mut buf);
        info.oz.write(&mut buf);

        let mut final_buf = BytesMut::with_capacity(5 + buf.len());
        final_buf.put_u8(0x0D);
        final_buf.put_u32(buf.len() as u32);
        final_buf.put_slice(&buf);

        writer.write().await.write_all(&final_buf).await?;
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
      let process = process_status_payload().await;

      let mut buf = BytesMut::new();
      buf.put_u8(0x0E);
      buf.put_u32(4);
      buf.put_u8(if PROCESS_ACTIVITY.load(Ordering::SeqCst) {
        0x01
      } else {
        0x02
      });
      buf.put_u8(process.status_id);
      buf.put_u8(process.connected_bots);
      buf.put_u8(process.total_bots);

      writer.write().await.write_all(&buf).await?;
    }
  }

  Ok(())
}
