# BeakId

`beakid` is a Snowflake-like 64-bit unique ID generator for Rust.

Bit layout:

```text
[ reserved: 1 | timestamp: 35 | sequence: 18 | worker: 10 ]
```

The timestamp unit is 100 milliseconds from a required custom epoch. The
reserved high bit is always `0`, so generated IDs fit in signed 64-bit storage.

## Configuration

The singleton generator reads configuration lazily from environment variables:

```env
BEAKID_EPOCH=2025-01-01T00:00:00Z
BEAKID_WORKER_ID=42
```

`BEAKID_EPOCH` is required and must be an RFC 3339 UTC datetime. `BEAKID_WORKER_ID`
defaults to `0` and must fit in 10 bits (`0..=1023`).

## Usage

```rust,no_run
fn main() -> Result<(), beakid::BeakIdError> {
    beakid::start_background()?;

    let id = beakid::try_next_id()?;
    assert_eq!(id >> 63, 0);

    Ok(())
}
```

For applications that prefer panics on invalid process configuration:

```rust,no_run
let id = beakid::next_id();
```

## Tokio

Enable the `tokio-rt` feature to spawn the background updater on a Tokio runtime:

```toml
beakid = { version = "0.1", features = ["tokio-rt"] }
```

```rust,ignore
#[tokio::main]
async fn main() -> Result<(), beakid::BeakIdError> {
    beakid::start_tokio_background()?;
    let id = beakid::try_next_id()?;
    println!("{id}");
    Ok(())
}
```

The feature also provides a declarative convenience macro:

```rust,ignore
beakid::tokio_main!({
    let id = beakid::try_next_id()?;
    println!("{id}");
    Ok::<(), Box<dyn std::error::Error>>(())
});
```

Rust attribute procedural macros must live in a separate `proc-macro` crate, so
this single-crate package exposes declarative macros and runtime helper
functions instead of `#[beakid::tokio_main]`.

## How It Works

The hot path uses atomics only. A background updater refreshes a real-time
100ms window hint roughly every 30ms, avoiding a system clock call for each ID.
When the per-window 18-bit sequence overflows, BeakId advances a virtual window
up to 10 windows ahead of the real hint. If that limit is reached, generation
waits until real time catches up.
