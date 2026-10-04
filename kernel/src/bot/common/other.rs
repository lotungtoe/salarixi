use azalea::registry::data::EnchantmentKey as Ek;

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

/// Функция конвертации ключа чары в строку
pub fn stringify_enchantment(key: Ek) -> String {
  match key {
    Ek::AquaAffinity => "aqua_affinity".to_string(),
    Ek::BaneOfArthropods => "bane_of_arthropods".to_string(),
    Ek::BindingCurse => "binding_curse".to_string(),
    Ek::BlastProtection => "blast_protection".to_string(),
    Ek::Breach => "breach".to_string(),
    Ek::Channeling => "channeling".to_string(),
    Ek::Density => "density".to_string(),
    Ek::DepthStrider => "depth_strider".to_string(),
    Ek::Efficiency => "efficiency".to_string(),
    Ek::FeatherFalling => "feather_falling".to_string(),
    Ek::FireAspect => "fire_aspect".to_string(),
    Ek::FireProtection => "fire_protection".to_string(),
    Ek::Flame => "flame".to_string(),
    Ek::Fortune => "fortune".to_string(),
    Ek::FrostWalker => "frost_walker".to_string(),
    Ek::Impaling => "impaling".to_string(),
    Ek::Infinity => "infinity".to_string(),
    Ek::Knockback => "knockback".to_string(),
    Ek::Looting => "looting".to_string(),
    Ek::Loyalty => "loyalty".to_string(),
    Ek::LuckOfTheSea => "luck_of_the_sea".to_string(),
    Ek::Lunge => "lunge".to_string(),
    Ek::Lure => "lure".to_string(),
    Ek::Mending => "mending".to_string(),
    Ek::Multishot => "multishot".to_string(),
    Ek::Piercing => "piercing".to_string(),
    Ek::Power => "power".to_string(),
    Ek::ProjectileProtection => "projectile_protection".to_string(),
    Ek::Protection => "protection".to_string(),
    Ek::Punch => "punch".to_string(),
    Ek::QuickCharge => "quick_charge".to_string(),
    Ek::Respiration => "respiration".to_string(),
    Ek::Riptide => "riptide".to_string(),
    Ek::Sharpness => "sharpness".to_string(),
    Ek::SilkTouch => "silk_touch".to_string(),
    Ek::Smite => "smite".to_string(),
    Ek::SoulSpeed => "soul_speed".to_string(),
    Ek::SweepingEdge => "sweeping_edge".to_string(),
    Ek::SwiftSneak => "swift_sneak".to_string(),
    Ek::Thorns => "thorns".to_string(),
    Ek::Unbreaking => "unbreaking".to_string(),
    Ek::VanishingCurse => "vanishing_curse".to_string(),
    Ek::WindBurst => "wind_burst".to_string(),
    Ek::Other(id) => id.to_string(),
  }
}
