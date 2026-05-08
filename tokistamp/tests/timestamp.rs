use tokistamp::{DateTime, Duration, Timestamp};

#[test]
fn creates_timestamp_from_milliseconds() {
    let timestamp = Timestamp::from_millis(123);

    assert_eq!(timestamp.as_i64(), 123);
}

#[test]
fn creates_timestamp_with_from_i64() {
    let timestamp = Timestamp::from(123_i64);

    assert_eq!(timestamp.as_i64(), 123);
}

#[test]
fn converts_date_time_to_timestamp() {
    let date_time = DateTime::parse("1970-01-02 00:00:00.123").unwrap();
    let timestamp = Timestamp::from(date_time);

    assert_eq!(timestamp.as_i64(), 86_400_123);
}

#[test]
fn formats_timestamp_as_date_time() {
    let timestamp = Timestamp::from_millis(86_400_123);

    assert_eq!(timestamp.to_string(), "1970-01-02 00:00:00.123");
}

#[test]
fn adds_duration_to_timestamp() {
    let timestamp = Timestamp::from_millis(1_000);

    assert_eq!(
        timestamp.add_duration(Duration::from_millis(234)).as_i64(),
        1_234
    );
    assert_eq!((timestamp + Duration::from_millis(234)).as_i64(), 1_234);
}

#[test]
fn subtracts_duration_from_timestamp() {
    let timestamp = Timestamp::from_millis(1_000);

    assert_eq!(
        timestamp.sub_duration(Duration::from_millis(234)).as_i64(),
        766
    );
    assert_eq!((timestamp - Duration::from_millis(234)).as_i64(), 766);
}

#[test]
fn subtracts_timestamp_as_duration() {
    let left = Timestamp::from_millis(1_500);
    let right = Timestamp::from_millis(250);

    assert_eq!((left - right).as_millis(), 1_250);
}
