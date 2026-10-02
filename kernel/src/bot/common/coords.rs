use azalea::Vec3;

// Функция получения средних координат ботов
pub fn get_average_coordinates_of_bots(positions: &Vec<Vec3>) -> (f64, f64, f64) {
  let mut x_coords = vec![];
  let mut y_coords = vec![];
  let mut z_coords = vec![];

  for pos in positions {
    x_coords.push(pos.x);
    y_coords.push(pos.x);
    z_coords.push(pos.z);
  }

  let mut x_global = 0.0;
  let mut y_global = 0.0;
  let mut z_global = 0.0;

  for coordinate in &x_coords {
    x_global = x_global + *coordinate;
  }

  for coordinate in &y_coords {
    y_global = y_global + *coordinate;
  }

  for coordinate in &z_coords {
    z_global = z_global + *coordinate;
  }

  let x_average = x_global / x_coords.len() as f64;
  let y_average = y_global / y_coords.len() as f64;
  let z_average = z_global / z_coords.len() as f64;

  (x_average, y_average, z_average)
}
