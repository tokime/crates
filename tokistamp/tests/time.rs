use std::str::FromStr;

use tokistamp::{Duration, Time};

#[test]
fn creates_time_from_hour_minute_string() {
    let time = Time::parse("12:34").unwrap();

    assert_eq!(time.hour(), 12);
    assert_eq!(time.minute(), 34);
    assert_eq!(time.second(), 0);
    assert_eq!(time.millisecond(), 0);
}

#[test]
fn creates_time_from_hour_minute_second_string() {
    let time = Time::parse("12:34:56").unwrap();

    assert_eq!(time.hour(), 12);
    assert_eq!(time.minute(), 34);
    assert_eq!(time.second(), 56);
    assert_eq!(time.millisecond(), 0);
}

#[test]
fn creates_time_from_hour_minute_second_millisecond_string() {
    let time = Time::parse("12:34:56.789").unwrap();

    assert_eq!(time.hour(), 12);
    assert_eq!(time.minute(), 34);
    assert_eq!(time.second(), 56);
    assert_eq!(time.millisecond(), 789);
}

#[test]
fn creates_time_from_date_time_string() {
    let time = Time::parse("2026-05-08 12:34:56.789").unwrap();

    assert_eq!(time.hour(), 12);
    assert_eq!(time.minute(), 34);
    assert_eq!(time.second(), 56);
    assert_eq!(time.millisecond(), 789);
}

#[test]
fn creates_time_with_from_str() {
    let time = Time::from_str("12:34:56.789").unwrap();

    assert_eq!(time.to_string(), "12:34:56.789");
}

#[test]
fn formats_time_with_milliseconds_even_when_zero() {
    let time = Time::parse("12:34:56").unwrap();

    assert_eq!(time.to_string(), "12:34:56.000");
}

#[test]
fn formats_time_with_milliseconds() {
    let time = Time::parse("12:34:56.007").unwrap();

    assert_eq!(time.to_string(), "12:34:56.007");
}

#[test]
fn formats_time_to_seconds() {
    let time = Time::parse("12:34:56.789").unwrap();

    assert_eq!(time.to_string_secs(), "12:34:56");
}

#[test]
fn formats_time_to_minutes() {
    let time = Time::parse("12:34:56.789").unwrap();

    assert_eq!(time.to_string_mins(), "12:34");
}

#[test]
fn adds_duration_to_time_from_unix_epoch_date() {
    let time = Time::parse("23:59:59.900").unwrap();
    let result = time.add_duration(Duration::from_millis(200));

    assert_eq!(result.to_string(), "1970-01-02 00:00:00.100");
}

#[test]
fn subtracts_duration_from_time_on_unix_epoch_date() {
    let time = Time::parse("00:00:00.100").unwrap();
    let result = time.sub_duration(Duration::from_millis(200));

    assert_eq!(result.to_string(), "1969-12-31 23:59:59.900");
}

#[test]
fn subtracts_time_as_duration() {
    let left = Time::parse("12:00:00.100").unwrap();
    let right = Time::parse("11:59:59.900").unwrap();

    assert_eq!((left - right).as_millis(), 200);
    assert_eq!((right - left).as_millis(), -200);
}

#[test]
fn rejects_invalid_time_string() {
    assert!(Time::parse("24:00").is_err());
    assert!(Time::parse("23:60").is_err());
    assert!(Time::parse("23:59:60").is_err());
    assert!(Time::parse("23:59:59.1000").is_err());
    assert!(Time::parse("23-59-59").is_err());
}

#[test]
fn now_creates_valid_time() {
    let time = Time::now();

    assert!(time.hour() <= 23);
    assert!(time.minute() <= 59);
    assert!(time.second() <= 59);
    assert!(time.millisecond() <= 999);
}
