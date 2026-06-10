/// Трейт плагина
pub trait SalarixiPlugin {
  fn new() -> Self;
  fn activate(&'static self, username: String) -> std::io::Result<()>;
}
