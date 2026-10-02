/// Функция конвертации индекса inventory-слота в индекс hotbar-слота
pub fn convert_inventory_slot_to_hotbar_slot(slot: usize) -> Option<u8> {
  match slot {
    36 => Some(0),
    37 => Some(1),
    38 => Some(2),
    39 => Some(3),
    40 => Some(4),
    41 => Some(5),
    42 => Some(6),
    43 => Some(7),
    44 => Some(8),
    _ => None,
  }
}

/// Функция конвертации индекса hotbar-слота в индекс inventory-слота
pub fn convert_hotbar_slot_to_inventory_slot(slot: u8) -> usize {
  match slot {
    0 => 36,
    1 => 37,
    2 => 38,
    3 => 39,
    4 => 40,
    5 => 41,
    6 => 42,
    7 => 43,
    8 => 44,
    _ => 36,
  }
}
