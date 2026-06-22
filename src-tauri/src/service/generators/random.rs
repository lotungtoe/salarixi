use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;

pub enum Classes {
  Numeric,
  Letter,
  Multi,
  Special,
}

/// Вспомогательная функция выбора случайного символа из строки
pub fn randchar(s: &str) -> char {
  let v: Vec<char> = s.chars().collect();
  let idx = thread_rng().gen_range(0..v.len());
  v[idx]
}

/// Функция генерации строки
pub fn randstr(class: Classes, length: i32) -> String {
  let chars = match class {
    Classes::Numeric => "0123456789",
    Classes::Letter => "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
    Classes::Multi => "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
    Classes::Special => "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_-+=",
  };

  let mut result = String::new();

  for _ in 0..length {
    result.push(randchar(chars));
  }

  result
}

/// Функция генерации случайного `int`
pub fn randint(min: i32, max: i32) -> i32 {
  thread_rng().gen_range(min..=max)
}

/// Функция генерации случайного `uint`
pub fn randuint(min: u64, max: u64) -> u64 {
  thread_rng().gen_range(min..=max)
}

/// Функция генерации случайного `float`
pub fn randfloat(min: f64, max: f64) -> f64 {
  thread_rng().gen_range(min..=max)
}

/// Функция выбора случайнного элемента из массива
pub fn randelem<T>(arr: &[T]) -> &T
where
  T: Sized,
{
  arr.choose(&mut thread_rng()).unwrap_or(&arr[0])
}

/// Функция генерации случайного шанса
pub fn randchance(percent: f64) -> bool {
  thread_rng().gen_bool(percent)
}
