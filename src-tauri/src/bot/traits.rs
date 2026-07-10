/// Трейт модуля
pub trait SalarixiModule<T> {
  /// Метод создания модуля
  fn new() -> Self;

  /// Метод изменения состояния модуля (включить / выключить)
  fn switch(&self, index: u8, options: T) -> impl std::future::Future<Output = bool> + Send + Sync;
}

/// Трейт плагина
pub trait SalarixiPlugin {
  /// Метод создания плагина
  fn new() -> Self;

  /// Метод активации плагина для определённого бота
  fn activate(&'static self, index: u8);
}
