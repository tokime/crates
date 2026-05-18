use std::time::{SystemTime, UNIX_EPOCH};

use tokistamp::{Date, DateTime, Datestamp, Duration};

const MILLIS_PER_DAY: u128 = 86_400_000;

#[test]
fn creates_datestamp_from_days() {
    let datestamp = Datestamp::from_days(2);

    assert_eq!(datestamp.as_i32(), 2);
}

#[test]
fn creates_datestamp_with_from_i32() {
    let datestamp = Datestamp::from(2_i32);

    assert_eq!(datestamp.as_i32(), 2);
}

#[test]
fn now_creates_current_datestamp() {
    let before = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / MILLIS_PER_DAY) as i32;
    let datestamp = Datestamp::now();
    let after = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / MILLIS_PER_DAY) as i32;

    assert!(datestamp.as_i32() >= before);
    assert!(datestamp.as_i32() <= after);
}

#[test]
fn converts_date_time_to_datestamp() {
    let date_time = DateTime::parse("1970-01-02 23:59:59.999").unwrap();
    let datestamp = Datestamp::from(date_time);

    assert_eq!(datestamp.as_i32(), 1);
}

#[test]
fn converts_date_to_datestamp() {
    let date = Date::parse("1970-01-03").unwrap();
    let datestamp = Datestamp::from(date);

    assert_eq!(datestamp.as_i32(), 2);
}

#[test]
fn formats_datestamp_as_date() {
    let datestamp = Datestamp::from_days(2);

    assert_eq!(datestamp.to_string(), "1970-01-03");
}

#[test]
fn adds_duration_to_datestamp_as_timestamp() {
    let datestamp = Datestamp::from_days(2);
    let timestamp = datestamp.add_duration(Duration::from_secs(5));

    assert_eq!(timestamp.as_i64(), 172_805_000);
    assert_eq!((datestamp + Duration::from_secs(5)).as_i64(), 172_805_000);
}

#[test]
fn subtracts_duration_from_datestamp_as_timestamp() {
    let datestamp = Datestamp::from_days(2);
    let timestamp = datestamp.sub_duration(Duration::from_secs(5));

    assert_eq!(timestamp.as_i64(), 172_795_000);
    assert_eq!((datestamp - Duration::from_secs(5)).as_i64(), 172_795_000);
}

#[test]
fn subtracts_datestamp_as_duration() {
    let left = Datestamp::from_days(3);
    let right = Datestamp::from_days(1);

    assert_eq!(
        (left - right).as_millis(),
        Duration::from_days(2).as_millis()
    );
}
