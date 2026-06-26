use azalea::swarm::*;

use crate::bot::registry::BOT_REGISTRY;

/// Обработчик для роя ботов
pub async fn swarm_handler(swarm: Swarm, event: SwarmEvent, _state: NoSwarmState) {
  match event {
    SwarmEvent::Init => {
      BOT_REGISTRY.set_swarm(swarm).await;
    }
    _ => {}
  }
}
