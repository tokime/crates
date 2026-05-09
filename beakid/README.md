# BeakId

English | [Русский](README.ru.md)

Snowflake-like 64-bit unique IDs for Rust. `BeakId` wraps a non-negative `i64`
value for PostgreSQL `BIGINT`.

```rust,no_run
fn main() -> Result<(), beakid::BeakIdError> {
    beakid::start_background()?;

    let id = beakid::try_next_id()?;
    assert!(id.as_i64() >= 0);

    Ok(())
}
```

## Layout

Each ID uses this 64-bit layout:

```text
[ reserved: 1 | timestamp: 35 | sequence: 18 | worker: 10 ]
```

Parts:

- `reserved`: always `0`
- `timestamp`: 100ms windows since the configured custom epoch
- `sequence`: per-window counter
- `worker`: generator worker id in `0..=1023`

Because the highest bit is always zero, generated IDs can be stored as valid
non-negative `i64` values.

## Configuration

The global singleton reads configuration lazily on first use:

```env
BEAKID_EPOCH=2025-01-01T00:00:00Z
BEAKID_WORKER_ID=42
```

Variables:

- `BEAKID_EPOCH`: required RFC 3339 UTC datetime, for example
  `2025-01-01T00:00:00Z`
- `BEAKID_WORKER_ID`: optional `u16`, defaults to `0`, must fit in 10 bits

Invalid or missing epoch configuration is rejected.

## API

### Singleton

```rust,no_run
let id = beakid::next_id();
```

`next_id()` panics if environment configuration is invalid. Use
`try_next_id()` to handle errors, including temporary generator blocking:

```rust,no_run
let id = beakid::try_next_id()?;
let db_id = id.as_i64();
# Ok::<(), beakid::BeakIdError>(())
```

Decode the absolute creation timestamp using the singleton epoch:

```rust,no_run
let id = beakid::try_next_id()?;
let created_at = beakid::timestamp(id)?;
# Ok::<(), beakid::BeakIdError>(())
```

### Background Updater

Start the standard-thread background updater once during application startup:

```rust,no_run
beakid::start_background()?;
# Ok::<(), beakid::BeakIdError>(())
```

The updater runs roughly every 30ms and reconciles the generator with real time.
The crate does not depend on Tokio or any other async runtime.

When the generator is blocked, `try_next_id()` returns `BeakIdError::Blocked`.
Async applications should retry through runtime-aware macros so the executor can
schedule other tasks while waiting:

```rust,ignore
let id = beakid::tokio_next_id!(generator);
let id = beakid::smol_next_id!(generator);
```

### Explicit Generator

```rust,no_run
use std::time::UNIX_EPOCH;

let generator = beakid::Generator::new(UNIX_EPOCH, 7)?;
let id = generator.next_id()?;
let created_at = generator.timestamp(id)?;
# Ok::<(), beakid::BeakIdError>(())
```

## Algorithm

`Generator` follows the reference `beakid-rs` approach:

```rust
pub struct Generator {
    id: std::sync::atomic::AtomicI64,
    state: std::sync::atomic::AtomicU64,
    epoch: std::time::SystemTime,
}
```

The hot path increments one atomic ID value by `1 << 10`, which advances the
sequence while preserving the worker bits. No mutexes are used.

When sequence overflow crosses a 100ms window boundary, the generator refreshes
real time. If the generated virtual window is ahead of real time, generation can
continue up to 10 virtual windows. If that limit is exhausted, generation returns
`BeakIdError::Blocked`. Waiting is intentionally left to runtime-aware macros or
caller code.

## Storage

Use PostgreSQL `BIGINT`:

```sql
id BIGINT PRIMARY KEY
```

Generated values are sortable by approximate creation time within the configured
epoch and worker-id scheme.

Use `id.as_i64()` before writing to the database, and `BeakId::new(value)` when
reading an ID back.
