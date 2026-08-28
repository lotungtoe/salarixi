use salarixi_macros::Index;

#[derive(Clone, Copy, Index)]
pub enum QuickTaskName {
  ClearInventory = 0x00,
  MoveForward = 0x01,
  MoveBackward = 0x02,
  MoveLeft = 0x03,
  MoveRight = 0x04,
  Jump = 0x05,
  Shift = 0x06,
  Fly = 0x07,
  Quit = 0x08,
  Rise = 0x09,
  Capsule = 0x0A,
  Unite = 0x0B,
  Turn = 0x0C,
  Zero = 0x0D,
  FormCircle = 0x0E,
  FormLine = 0x0F,
  StopPathfinder = 0x10,
  Reset = 0x11,
}

impl QuickTaskName {
  pub fn to_str(&self) -> &str {
    match self {
      Self::ClearInventory => "clear-inventory",
      Self::MoveForward => "move-forward",
      Self::MoveBackward => "move-backward",
      Self::MoveLeft => "move-left",
      Self::MoveRight => "move-right",
      Self::Jump => "jump",
      Self::Shift => "shift",
      Self::Fly => "fly",
      Self::Quit => "quit",
      Self::Rise => "rise",
      Self::Capsule => "capsule",
      Self::Unite => "unite",
      Self::Turn => "turn",
      Self::Zero => "zero",
      Self::FormCircle => "form-circle",
      Self::FormLine => "form-line",
      Self::StopPathfinder => "stop-pathfinder",
      Self::Reset => "reset",
    }
  }
}
