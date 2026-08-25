# Расширенный скриптинг

Здесь мы подробно рассмотрим написание сценариев для ботов, изучим реальные примеры с использованием основных функций, расскажем о различных исключениях, что стоит делать и что нет.

# Базовые понятия

Все сценарии в клиенте пишутся на ограниченном Rust. Движок, исполняющий сценарий, называется Rhai.

Хоть Rust и ограниченный, но многие из его основных возможностей доступны:

- Объявление переменных
- Изменение значения в переменных (но без привычной концепции `mut`)
- Объявление констант
- Объявление и вызов функций
- Циклы `for` и `while`
- Операторы `+`, `*`, `-`, `/`, `+=`, `!`, `>`, `<`, `=`, `!=`, и так далее
- Условные ветвления `if-else` и `switch` (аналог `match`)
- Логические операторы `||`, `&&`
- Комментарии

Для написания сценариев под ботов этого достаточно.

Главный плюс Rhai - динамическая типизация. Все данные в сценарии фактически не имеют какого-то строго заданного типа как в Rust, всё обёрнуто в Dynamic. Функция может вернуть как `0`, так и `"lol"`.

Подробнее об Rhai можно почитать здесь: https://rhai.rs/book/about/index.html

# Перед написанием сценариев

1. Не рекомендуется ставить циклы рода `while true`, в теле которых выполняются долгие операции, так как такой сценарий будет трудно остановить.

2. Клиент не понимает, какие состояния были изменены вашим сценарием, соотвественно после его остановки/завершения боты продолжат заданные действия (например ходьбу). Поэтому стоит продумывать логику сценария так, чтобы в конце он сам выключал активные состояния.

# Основной функционал

Здесь я не буду лишний раз дублировать все встроенные функции сценария для работы с ботами, их можно посмотреть на https://salarixi.freedev.app/pages/documentation.html в разделе **Сценарий**.

## Логгирование действий

Всего есть 4 функции, позволяющие передать текстовую информацию куда-либо: `log`, `message`, `webhook`, `print`.

Для простого логгирования действий (короткие сообщения) лучше всего использовать функцию `log`, она удобна тем, что позволяет выбрать тип лога (`info`, `warning` и т.п.).

Рассмотрим простой пример:

```
// Попробуй запустить сценарий где 'n' равняется 0,
// и где `n` равняется любому числу, отличному от 0
let n = 0;

if n == 0 {
  log("[script]: Переменная 'n' равняется 0", "info");
} else {
  log("[script]: Переменная 'n' не равняется 0", "error");
}
```

## Управление чатом

Есть только одна функция, позволяющая отправлять сообщения в чат от ботов: `chat`.

Пример отправки 3 сообщений от всех ботов:

```
for i in 0..3 {
  chat("*", "Hello, world (repeat=" + i + ")");
  sleep(1.5);
}
```

## Управление движением

Здесь на выбор даётся несколько функций: `start_walking`, `start_sprinting`, `start_pathfinder`. Чтобы останавливать движение, можно использовать: `stop_move`, `stop_pathfinder`.

Простой пример использования `start_walking` + `stop_move`:

```
fn walk(direction) {
  start_walking("*", direction);
  sleep(2);
  stop_move("*");
}

let directions = [
  "forward",
  "left",
  "forward-right",
  "backward-right",
];

// Повтор 0: "forward"
// Повтор 1: "left"
// Повтор 2: "forward-right"
// Повтор 3: "backward-right"
for i in 0..4 {
  walk(directions[i]);
  sleep(1);
}
```

## Управление инвентарём

Сценарий содержит в себе кучу встроенных функций для управления инвентарём: `inv_click`, `inv_click_on`, `set_selected_slot`, `drop_selected_item`, `inv_extract_all`, `inv_extract_by_slot`, `inv_extract_by_name`.

Пример использования `inv_extract_all` + `inv_click`:

```
// Сюда подставь юзернейм бота
let username = "...";

// Получаем все предметы из инвентаря бота
let items = inv_extract_all(username);

// Проходимся по предметам, выбрасываем
// все предметы кроме золотых яблок
for item in items {
  if item["kind"] == "minecraft:golden_apple" {
    continue;
  }

  inv_click(username, item["slot"], "drop-all", false);
}

// Скорее всего ты думаешь, зачем такой код когда можно
// использовать уже готовые функции типа inv_click_on,
// и inv_extract_by_name. Ответ - чтобы глубже понять,
// как работать с инвентарём и перебором предметов.
```

Ещё простой пример использования `inv_click_on`:

```
// Выбрасываем все алмазы из первого попавшегося слота с ними
inv_click_on("*", "Diamond", "drop-all", false);
```

## Взаимодействие с предметами

Чтобы взаимодействовать с предметами (например, натянуть лук, нажать на компас), используются функции: `start_use_item`, `release_use_item`.

Вот пример, где подразумевается, что у ботов в основной руке лук и есть стрелы:

```
start_use_item("*", "mainhand");
sleep(2);
release_use_item("*");
```

# Распределение состояний

В клиенте есть уникальная система состояний ботов, которая позволяет избегать конфликты между разными действиями бота. Все плагины и модули управления стараются взаимно устанавливать эти состояния, чтобы бот мог одновременно выполнять несколько задач.

Те же возможности есть в сценарии, это всего лишь две функции: `set_state`, `get_state`.

Почти все функции в сценарии автоматически распределяют эти состояния (где-то есть аргумент `lock`, что позволяет конкретизировать, будет ли функция опираться на состояния).

Пример блокировки состояния:

```
// Сюда подставь юзернейм бота
let username = "...";

// Попробуй закомментировать строку ниже
set_state(username, "can_walking", false);
start_walking(username, "forward");
```

# Сценарии для реальных проблем

Наконец мы можем приступить к рассмотрению сценариев, которые решают реальные задачи. Код, возможно, с первого раза будет понять трудно - это нормально.

## Экипировка доспехов

Конечно, в клиенте есть умный плагин **Instant Armor Equip**, который решает эту проблему. Но мы всё равно напишем упрощённую версию плагина для изучения:

```
// Сюда подставь юзернейм бота
let username = "...";

let items = inv_extract_all(username);

for item in items {
  let helmet_equipped = inv_extract_by_slot(username, 5)["kind"] != "minecraft:air";
  let chestplate_equipped = inv_extract_by_slot(username, 6)["kind"] != "minecraft:air";
  let leggings_equipped = inv_extract_by_slot(username, 7)["kind"] != "minecraft:air";
  let boots_equipped = inv_extract_by_slot(username, 8)["kind"] != "minecraft:air";

  let should_equip = false;

  switch item["kind"][10..] {
    "netherite_helmet" |
    "diamond_helmet" |
    "iron_helmet" |
    "copper_helmet" |
    "chainmail_helmet" |
    "golden_helmet" |
    "turtle_shell" |
    "leather_cap" => {
      should_equip = !helmet_equipped;
    }
    "netherite_hestplate" |
    "diamond_chestplate" |
    "iron_chestplate" |
    "copper_chestplate" |
    "chainmail_chestplate" |
    "golden_chestplate" |
    "leather_tunic" => {
      should_equip = !chestplate_equipped;
    }
    "netherite_leggings" |
    "diamond_leggings" |
    "iron_leggings" |
    "copper_leggings" |
    "chainmail_leggings" |
    "golden_leggings" |
    "leather_pants" => {
      should_equip = !leggings_equipped;
    }
    "netherite_boots" |
    "diamond_boots" |
    "iron_boots" |
    "copper_boots" |
    "chainmail_boots" |
    "golden_boots" |
    "leather_boots" => {
      should_equip = !boots_equipped;
    }
    _ => {}
  }

  if should_equip {
    inv_click(username, item["slot"], "shift", false);
  }
}
```

После запуска сценария выше, боты должны экипировать существующую броню. Этот сценарий не сравнивает броню по чарам, материалу и так далее.

## Очистка инвентаря от мусора

```
// Сюда подставь юзернейм бота
let username = "...";

// Сюда можешь поставить свои значения
let trash = [
  "dirt",
  "grass_block",
  "stick",
];

let items = inv_extract_all(username);

for item in items {
  if !trash.contains(item["kind"][10..]) {
    return;
  }

  inv_click(username, item["slot"], "drop-all", false);
}
```
