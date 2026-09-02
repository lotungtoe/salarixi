pub trait ModuleOptions
where
  Self: Sized,
{
  fn from_bytes(buf: &mut bytes::Bytes) -> Option<Self>;
}

pub trait SalarixiModule<O: ModuleOptions> {
  fn new() -> Self;
  fn switch(&self, index: u8, options: std::sync::Arc<O>) -> impl std::future::Future<Output = bool> + Send + Sync;
}

pub trait SalarixiPlugin<O> {
  fn new() -> Self;
  fn activate(&'static self, index: u8, options: std::sync::Arc<O>);
}
