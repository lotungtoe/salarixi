use azalea::swarm::*;
use azalea::NoState;

use crate::bot::systems::index::INDEX_SYSTEM;
use crate::bot::systems::registry::REGISTRY_SYSTEM;
use crate::launch::process::current_options;
use crate::sleep;

pub async fn swarm_handler(swarm: Swarm, event: SwarmEvent, _state: NoSwarmState) {
  match event {
    SwarmEvent::Init => {
      REGISTRY_SYSTEM.set_swarm(swarm).await;
    }
    SwarmEvent::Disconnect(account, join_opts) => {
      let Some(opts) = current_options().await else {
        return;
      };

      if opts.basic.use_auto_rejoin {
        let Some(index) = INDEX_SYSTEM.index_by_username(account.username()).await else {
          return;
        };

        sleep!(opts.basic.rejoin_delay as u64);
        let bot = swarm.add_with_opts(&account, NoState, &join_opts).await;
        REGISTRY_SYSTEM.register_bot(index, bot);
      }
    }
    _ => {}
  }
}
