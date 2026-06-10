/// Трейт плагина
pub trait SalarixiPlugin {
  /// Метод создания плагина
  fn new() -> Self;

  /// Метод активации плагина для определённого бота
  fn activate(&'static self, username: String) -> std::io::Result<()>;
}
