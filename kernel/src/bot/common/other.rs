use crate::bot::extensions::BotDefaultExt;
use crate::bot::systems::profile::PROFILE_SYSTEM;
use crate::bot::systems::registry::REGISTRY_SYSTEM;

/// Функция получения UUID игрока
pub async fn get_player_uuid(username: String) -> Option<String> {
  for index in PROFILE_SYSTEM.get_all_connected().await.keys() {
    let uuid = REGISTRY_SYSTEM
      .async_get_bot(index, async |bot| {
        let Some(tab_list) = bot.get_players() else {
          return None;
        };

        for (uuid, info) in tab_list.iter() {
          if info.profile.name == username {
            return Some(uuid.to_string());
          }
        }

        None
      })
      .await;

    if let Some(u) = uuid {
      return u;
    }
  }

  None
}
