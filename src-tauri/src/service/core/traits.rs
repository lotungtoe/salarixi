/// Трейт модуля
pub trait SalarixiModule<T> {
  /// Метод создания модуля
  fn new() -> Self;

  /// Метод изменения состояния модуля (включить / выключить)
  fn switch(&self, username: String, options: T) -> bool;
}

/// Трейт плагина
pub trait SalarixiPlugin {
  /// Метод создания плагина
  fn new() -> Self;

  /// Метод активации плагина для определённого бота
  fn activate(&'static self, username: String) -> std::io::Result<()>;
}
