use azalea::block::BlockState;
use azalea::{BlockPos, Client};

/// Функция получения состояния блока
pub fn get_block_state(bot: &Client, block_pos: BlockPos) -> Option<BlockState> {
  if let Some(state) = bot.world().read().get_block_state(block_pos) {
    return Some(state);
  }

  None
}
