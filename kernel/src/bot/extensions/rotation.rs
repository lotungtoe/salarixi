use azalea::BlockPos;
use azalea::Client;
use azalea::EntityRef;
use azalea::Vec3;

use crate::bot::extensions::BotDefaultExt;
use crate::sleep;
use crate::tools::{randchance, randnum};

pub trait BotRotationExt {
  fn look_at_entity(&self, entity: &EntityRef, with_eye_height: bool);
  fn look_at_block(&self, block_pos: BlockPos, sloppy: bool) -> impl std::future::Future<Output = ()> + Send + Sync;
}

impl BotRotationExt for Client {
  fn look_at_entity(&self, entity: &EntityRef, with_eye_height: bool) {
    let Some(mut entity_pos) = self.get_entity_position(entity) else {
      return;
    };

    if with_eye_height {
      let Some(eye_height) = self.get_entity_eye_height(entity) else {
        return;
      };

      entity_pos.y += eye_height;
    }

    self.look_at(Vec3 {
      x: entity_pos.x,
      y: entity_pos.y,
      z: entity_pos.z,
    });
  }

  async fn look_at_block(&self, block_pos: BlockPos, sloppy: bool) {
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
  }
}
