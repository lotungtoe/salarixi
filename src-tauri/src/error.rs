/// Вспомогательный макрос создания ошибки
#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => {
    return Err(std::io::Error::other(format!($($arg)*)))
  };
}
