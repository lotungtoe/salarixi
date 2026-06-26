use super::{ping_server, ServerInformation};

/// Команда пингования сервера
#[tauri::command]
pub async fn get_server_info(address: String) -> ServerInformation {
  ping_server(address).await
}
