use azalea::core::direction::Direction;
use azalea::ecs::entity::Entity;
use azalea::protocol::packets::game::s_interact::InteractionHand;
use azalea::protocol::packets::game::s_player_action::Action;
use azalea::protocol::packets::game::{ServerboundPlayerAction, ServerboundUseItem};
use azalea::{BlockPos, Client};

use crate::bot::systems::states::{getst, setmst, setst, State, StateName};

pub trait BotInteractExt {
  fn start_use_item_by(&self, hand: InteractionHand);
  fn release_use_item(&self);
  fn attack_entity(&self, index: &u8, entity: Entity) -> impl std::future::Future<Output = ()> + Send + Sync;
}

impl BotInteractExt for Client {
  fn start_use_item_by(&self, hand: InteractionHand) {
    let direction = self.direction();

    self.write_packet(ServerboundUseItem {
      hand: hand,
      y_rot: direction.0,
      x_rot: direction.1,
      seq: 0,
    });
  }

  fn release_use_item(&self) {
    self.write_packet(ServerboundPlayerAction {
      action: Action::ReleaseUseItem,
      pos: BlockPos::new(0, 0, 0),
      direction: Direction::Down,
      seq: 0,
    });
  }

  async fn attack_entity(&self, index: &u8, entity: Entity) {
    if !getst(index, State::CanAttacking).await {
      return;
    }

    setst(index, State::CanDrinking, false).await;
    setst(index, State::CanEating, false).await;
    setst(index, State::CanInteracting, false).await;
    setmst(index, StateName::Attacking, true).await;

    self.attack(entity);

    setmst(index, StateName::Attacking, false).await;
    setst(index, State::CanDrinking, true).await;
    setst(index, State::CanEating, true).await;
    setst(index, State::CanInteracting, true).await;
  }
}
