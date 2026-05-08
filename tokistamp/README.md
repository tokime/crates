# tokistamp

English | [Русский](README.ru.md)

Small UTC date, time, and Unix epoch types without timezone handling.

```rust
use tokistamp::{Datestamp, Timestamp};

let timestamp = Timestamp::from_millis(86_400_123);
let timestamp_text = timestamp.to_string();
assert_eq!(timestamp_text, "1970-01-02 00:00:00.123");

let datestamp = Datestamp::from_days(2);
let datestamp_text = datestamp.to_string();
assert_eq!(datestamp_text, "1970-01-03");
```

## Types

### `DateTime`

Stores a UTC date and time with millisecond precision.

Fields are private, so invalid state cannot be created directly. Use
`DateTime::new(...)`, `DateTime::parse(...)`, `DateTime::now()`, or `FromStr`.

Supported parse formats:

- `yyyy-MM-dd HH:mm:ss.SSS`
- `yyyy-MM-dd HH:mm:ss`
- `yyyy-MM-dd HH:mm`
- `yyyy-MM-dd`
- `HH:mm:ss.SSS`
- `HH:mm:ss`
- `HH:mm`

`Display` always returns:

- `yyyy-MM-dd HH:mm:ss.SSS`

Additional formats:

- `to_string_secs()` -> `yyyy-MM-dd HH:mm:ss`
- `to_string_mins()` -> `yyyy-MM-dd HH:mm`

### `Date`

Stores a UTC date without time.

Created through `DateTime` narrowing: `Date::parse(...)`, `DateTime::date()`,
`Date::now()`, or `FromStr`.

`Display` returns:

- `yyyy-MM-dd`

### `Time`

Stores a UTC time of day with millisecond precision.

Created through `DateTime` narrowing: `Time::parse(...)`, `DateTime::time()`,
`Time::now()`, or `FromStr`.

`Display` always returns:

- `HH:mm:ss.SSS`

Additional formats:

- `to_string_secs()` -> `HH:mm:ss`
- `to_string_mins()` -> `HH:mm`

### `Duration`

Stores a duration in milliseconds: `pub struct Duration(i64)`.

Creation:

- `Duration::from_millis(...)`
- `Duration::from_secs(...)`
- `Duration::from_mins(...)`
- `Duration::from_hours(...)`
- `Duration::from_days(...)`

`Display` returns only non-zero parts:

- `12d 23h 45m 56s 123ms`
- `1d 2s`
- `123ms`
- `0ms`

### `Timestamp`

Stores Unix epoch time in milliseconds: `pub struct Timestamp(i64)`.

Creation and value access:

- `Timestamp::from_millis(...)`
- `Timestamp::from(i64)`
- `as_i64()`

`Display` returns the same format as `DateTime`:

- `yyyy-MM-dd HH:mm:ss.SSS`

Conversions:

- `DateTime -> Timestamp`

### `Datestamp`

Stores Unix epoch date in days: `pub struct Datestamp(i32)`.

Creation and value access:

- `Datestamp::from_days(...)`
- `Datestamp::from(i32)`
- `as_i32()`

`Display` returns the same format as `Date`:

- `yyyy-MM-dd`

Conversions:

- `DateTime -> Datestamp`
- `Date -> Datestamp`

## Math

`DateTime`, `Date`, `Time`, `Timestamp`, and `Datestamp` support `Duration`
operations through `add_duration(...)` and `sub_duration(...)`.

Available operators:

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
