# BeakId

[English](README.md) | Русский

Snowflake-подобные 64-битные уникальные ID для Rust. `BeakId` оборачивает
неотрицательное `i64`, поэтому значение можно хранить в PostgreSQL `BIGINT`.

```rust,no_run
fn main() -> Result<(), beakid::BeakIdError> {
    beakid::start_background()?;

    let id = beakid::try_next_id()?;
    assert!(id.as_i64() >= 0);

    Ok(())
}
```

## Раскладка

Каждый ID использует такую 64-битную раскладку:

```text
[ reserved: 1 | timestamp: 35 | sequence: 18 | worker: 10 ]
```

Части:

- `reserved`: всегда `0`
- `timestamp`: окна по 100 мс от настроенной custom epoch
- `sequence`: счётчик внутри окна
- `worker`: worker id генератора в диапазоне `0..=1023`

Так как старший бит всегда равен нулю, все сгенерированные ID можно хранить как
неотрицательные `i64`.

## Конфигурация

Глобальный singleton читает конфигурацию лениво при первом использовании:

```env
BEAKID_EPOCH=2025-01-01T00:00:00Z
BEAKID_WORKER_ID=42
```

Переменные:

- `BEAKID_EPOCH`: обязательная UTC дата-время в RFC 3339, например
  `2025-01-01T00:00:00Z`
- `BEAKID_WORKER_ID`: необязательный `u16`, по умолчанию `0`, должен помещаться
  в 10 бит

Некорректная или отсутствующая epoch отклоняется.

## API

### Singleton

```rust,no_run
let id = beakid::next_id();
```

`next_id()` паникует при некорректной конфигурации окружения. Если ошибки нужно
обработать явно, включая временную блокировку генератора, используйте
`try_next_id()`:

```rust,no_run
let id = beakid::try_next_id()?;
let db_id = id.as_i64();
# Ok::<(), beakid::BeakIdError>(())
```

Получение абсолютного времени создания через epoch singleton-генератора:

```rust,no_run
let id = beakid::try_next_id()?;
let created_at = beakid::timestamp(id)?;
# Ok::<(), beakid::BeakIdError>(())
```

### Фоновое обновление

Запустите стандартный фоновый поток один раз при старте приложения:

```rust,no_run
beakid::start_background()?;
# Ok::<(), beakid::BeakIdError>(())
```

Фоновый поток работает примерно каждые 30 мс и синхронизирует генератор с
реальным временем. Крейт не зависит от Tokio или другого async runtime.

Когда генератор заблокирован, `try_next_id()` возвращает
`BeakIdError::Blocked`. Async-приложениям лучше повторять генерацию через
runtime-aware макросы, чтобы executor мог выполнять другие задачи во время
ожидания:

```rust,ignore
let id = beakid::tokio_next_id!();
let id = beakid::smol_next_id!();
```

### Явный генератор

```rust,no_run
use std::time::UNIX_EPOCH;

let generator = beakid::Generator::new(UNIX_EPOCH, 7)?;
let id = generator.next_id()?;
let created_at = generator.timestamp(id)?;
# Ok::<(), beakid::BeakIdError>(())
```

## Алгоритм

`Generator` следует подходу из reference `beakid-rs`:

```rust
pub struct Generator {
    id: std::sync::atomic::AtomicI64,
    state: std::sync::atomic::AtomicU64,
    epoch: std::time::SystemTime,
}
```

Горячий путь увеличивает одно атомарное значение ID на `1 << 10`. Это двигает
`sequence`, сохраняя младшие биты `worker`. Mutex не используется.

Когда переполнение `sequence` переводит генератор в следующее 100мс окно,
генератор обновляет реальное время. Если виртуальное окно оказалось впереди
реального времени, генерация продолжается максимум на 10 виртуальных окон. Если
лимит исчерпан, генерация возвращает `BeakIdError::Blocked`. Ожидание намеренно
оставлено runtime-aware макросам или коду вызывающей стороны.

## Хранение

Используйте PostgreSQL `BIGINT`:

```sql
id BIGINT PRIMARY KEY
```

Сгенерированные значения сортируются примерно по времени создания в рамках
выбранной epoch и схемы worker id.

Используйте `id.as_i64()` перед записью в базу и `BeakId::new(value)` при чтении
ID обратно.
