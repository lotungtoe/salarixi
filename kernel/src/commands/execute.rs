use std::sync::Arc;

use aes_gcm::Aes256Gcm;
use bytes::Bytes;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::RwLock;

use crate::commands::enumeration::ClientCommand;
use crate::server::Sockets;

use super::impls::*;

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
