use azalea::pathfinder::astar::PathfinderTimeout;
use azalea::pathfinder::goals::XZGoal;
use azalea::pathfinder::moves::basic::basic_move;
use azalea::pathfinder::PathfinderOpts;
use azalea::{prelude::*, SprintDirection, StartSprintEvent, StartWalkEvent, WalkDirection};
use std::time::Duration;

use crate::service::core::controllers::{getst, setmst, setst, State, StateName};
use crate::take_bot;

pub trait BotMovementExt {
  fn start_walking(&self, direction: WalkDirection) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn start_sprinting(&self, direction: SprintDirection) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn stop_move(&self) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn freeze_move(&self) -> impl std::future::Future<Output = ()> + Send + Sync;
  fn unfreeze_move(&self) -> impl std::future::Future<Output = ()> + Send + Sync;
}

impl BotMovementExt for Client {
  async fn start_walking(&self, direction: WalkDirection) {
    let username = self.username();

    if getst(&username, State::CanWalking).await
      && !getst(&username, State::IsEating).await
      && !getst(&username, State::IsDrinking).await
      && !getst(&username, State::IsInteracting).await
    {
      self.ecs.lock().write_message(StartWalkEvent {
        entity: self.entity,
        direction: direction,
      });

      setmst(&username, StateName::Walking, true).await;
    }
  }

  async fn start_sprinting(&self, direction: SprintDirection) {
    let username = self.username();

    if getst(&username, State::CanSprinting).await
      && !getst(&username, State::IsEating).await
      && !getst(&username, State::IsDrinking).await
      && !getst(&username, State::IsInteracting).await
    {
      self.ecs.lock().write_message(StartSprintEvent {
        entity: self.entity,
        direction: direction,
      });

      setmst(&username, StateName::Sprinting, true).await;
    }
  }

  async fn stop_move(&self) {
    self.ecs.lock().write_message(StartWalkEvent {
      entity: self.entity,
      direction: WalkDirection::None,
    });

    let username = self.username();

    setmst(&username, StateName::Walking, false).await;
    setmst(&username, StateName::Sprinting, false).await;
  }

  async fn freeze_move(&self) {
    self.ecs.lock().write_message(StartWalkEvent {
      entity: self.entity,
      direction: WalkDirection::None,
    });

    let username = self.username();

    setst(&username, State::IsWalking, false).await;
    setst(&username, State::IsSprinting, false).await;
    setst(&username, State::CanWalking, false).await;
    setst(&username, State::CanSprinting, false).await;
  }

  async fn unfreeze_move(&self) {
    let username = self.username();

    setst(&username, State::CanWalking, true).await;
    setst(&username, State::CanSprinting, true).await;
  }
}

/// Функция безопасного задания координат по X и Z для бота
pub fn go_to(username: String, x: i32, z: i32) {
  tokio::spawn(async move {
    if !getst(&username, State::CanWalking).await || !getst(&username, State::CanSprinting).await {
      return;
    }

    take_bot!(&username, async |bot| {
      setmst(&username, StateName::Looking, true).await;
      setmst(&username, StateName::Sprinting, true).await;
      setmst(&username, StateName::Walking, true).await;

      let goal = XZGoal { x: x, z: z };
      let opts = PathfinderOpts::new()
        .min_timeout(PathfinderTimeout::Time(Duration::from_millis(500)))
        .max_timeout(PathfinderTimeout::Time(Duration::from_millis(1000)))
        .allow_mining(false)
        .successors_fn(basic_move);

      bot.goto_with_opts(goal, opts).await;

      setmst(&username, StateName::Looking, false).await;
      setmst(&username, StateName::Sprinting, false).await;
      setmst(&username, StateName::Walking, false).await;
    });
  });
}
