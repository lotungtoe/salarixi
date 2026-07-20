use azalea::prelude::*;
use azalea::BlockPos;
use azalea::Client;
use azalea::Vec3;
use bevy_ecs::entity::Entity;

use crate::bot::extensions::BotDefaultExt;
use crate::bot::systems::states::setst;
use crate::bot::systems::states::{getst, setmst, State, StateName};
use crate::other::tools::{randchance, randnum};
use crate::sleep;

pub trait BotRotationExt {
  fn look_at_entity(
    &self,
    index: &u8,
    entity: Entity,
    with_eye_height: bool,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn look_at_block(
    &self,
    index: &u8,
    block_pos: BlockPos,
    sloppy: bool,
    lock: bool,
  ) -> impl std::future::Future<Output = ()> + Send + Sync;
}

impl BotRotationExt for Client {
  async fn look_at_entity(&self, index: &u8, entity: Entity, with_eye_height: bool, lock: bool) {
    if lock {
      if !getst(index, State::CanLooking).await {
        return;
      }

      setst(index, State::CanInteracting, false).await;
      setmst(index, StateName::Looking, true).await;
    }

    let mut entity_pos = self.get_entity_position(entity);

    if with_eye_height {
      entity_pos.y += self.get_entity_eye_height(entity);
    }

    self.look_at(Vec3 {
      x: entity_pos.x,
      y: entity_pos.y,
      z: entity_pos.z,
    });

    if lock {
      setmst(index, StateName::Looking, false).await;
      setst(index, State::CanInteracting, true).await;
    }
  }

  async fn look_at_block(&self, index: &u8, block_pos: BlockPos, sloppy: bool, lock: bool) {
    if lock {
      if !getst(index, State::CanLooking).await {
        return;
      }

      setst(index, State::CanInteracting, false).await;
      setmst(index, StateName::Looking, true).await;
    }

    let center = block_pos.center();

    if sloppy && randchance(0.4) {
      let sloppy_pos = Vec3::new(
        center.x as f64 + randnum(-1.3289, 1.3289),
        center.y as f64 + randnum(-1.3289, 1.3289),
        center.z as f64 + randnum(-1.3289, 1.3289),
      );

      self.look_at(sloppy_pos);

      sleep!(100);
    }

    self.look_at(Vec3 {
      x: center.x,
      y: center.y,
      z: center.z,
    });

    if lock {
      setmst(index, StateName::Looking, false).await;
      setst(index, State::CanInteracting, true).await;
    }
  }
}
