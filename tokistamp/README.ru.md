# tokistamp

[English](README.md) | Русский

Небольшие типы для работы с UTC датой, временем и Unix epoch без timezone.

```rust
use tokistamp::{Datestamp, Timestamp};

let timestamp = Timestamp::from_millis(86_400_123);
let timestamp_text = timestamp.to_string();
assert_eq!(timestamp_text, "1970-01-02 00:00:00.123");

let datestamp = Datestamp::from_days(2);
let datestamp_text = datestamp.to_string();
assert_eq!(datestamp_text, "1970-01-03");
```

## Структуры

### `DateTime`

Хранит UTC дату и время с точностью до миллисекунд.

Поля закрыты, создать некорректное состояние нельзя. Создание доступно через
`DateTime::new(...)`, `DateTime::parse(...)`, `DateTime::now()` и `FromStr`.

Поддерживаемые форматы парсинга:

- `yyyy-MM-dd HH:mm:ss.SSS`
- `yyyy-MM-dd HH:mm:ss`
- `yyyy-MM-dd HH:mm`
- `yyyy-MM-dd`
- `HH:mm:ss.SSS`
- `HH:mm:ss`
- `HH:mm`

`Display` всегда выводит:

- `yyyy-MM-dd HH:mm:ss.SSS`

Дополнительные форматы:

- `to_string_secs()` -> `yyyy-MM-dd HH:mm:ss`
- `to_string_mins()` -> `yyyy-MM-dd HH:mm`

### `Date`

Хранит UTC дату без времени.

Создаётся через `DateTime` и сужение: `Date::parse(...)`, `DateTime::date()`,
`Date::now()` и `FromStr`.

`Display` выводит:

- `yyyy-MM-dd`

### `Time`

Хранит UTC время суток с точностью до миллисекунд.

Создаётся через `DateTime` и сужение: `Time::parse(...)`, `DateTime::time()`,
`Time::now()` и `FromStr`.

`Display` всегда выводит:

- `HH:mm:ss.SSS`

Дополнительные форматы:

- `to_string_secs()` -> `HH:mm:ss`
- `to_string_mins()` -> `HH:mm`

### `Duration`

Хранит длительность в миллисекундах: `pub struct Duration(i64)`.

Создание:

- `Duration::from_millis(...)`
- `Duration::from_secs(...)`
- `Duration::from_mins(...)`
- `Duration::from_hours(...)`
- `Duration::from_days(...)`

`Display` выводит только ненулевые части:

- `12d 23h 45m 56s 123ms`
- `1d 2s`
- `123ms`
- `0ms`

### `Timestamp`

Хранит время от Unix epoch в миллисекундах: `pub struct Timestamp(i64)`.

Создание и получение значения:

- `Timestamp::from_millis(...)`
- `Timestamp::from(i64)`
- `as_i64()`

`Display` выводит как `DateTime`:

- `yyyy-MM-dd HH:mm:ss.SSS`

Конверсии:

- `DateTime -> Timestamp`

### `Datestamp`

Хранит дату от Unix epoch в днях: `pub struct Datestamp(i32)`.

Создание и получение значения:

- `Datestamp::from_days(...)`
- `Datestamp::from(i32)`
- `as_i32()`

`Display` выводит как `Date`:

- `yyyy-MM-dd`

Конверсии:

- `DateTime -> Datestamp`
- `Date -> Datestamp`

## Математика

`DateTime`, `Date`, `Time`, `Timestamp` и `Datestamp` поддерживают операции с
`Duration` через методы `add_duration(...)` и `sub_duration(...)`.

Также доступны операторы:

- `Duration + Duration -> Duration`
- `Duration - Duration -> Duration`
- `DateTime - DateTime -> Duration`
- `Date - Date -> Duration`
- `Time - Time -> Duration`
- `Timestamp - Timestamp -> Duration`
- `Datestamp - Datestamp -> Duration`
- `Timestamp + Duration -> Timestamp`
- `Timestamp - Duration -> Timestamp`
- `Datestamp + Duration -> Timestamp`
- `Datestamp - Duration -> Timestamp`
