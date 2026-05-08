use std::str::FromStr;

use tokistamp::{DateTime, Duration};

#[test]
fn creates_date_time_from_checked_fields() {
    let date_time = DateTime::new(2026, 5, 8, 12, 34, 56, 789).unwrap();

    assert_eq!(date_time.year(), 2026);
    assert_eq!(date_time.month(), 5);
    assert_eq!(date_time.day(), 8);
    assert_eq!(date_time.hour(), 12);
    assert_eq!(date_time.minute(), 34);
    assert_eq!(date_time.second(), 56);
    assert_eq!(date_time.millisecond(), 789);
}

#[test]
fn creates_date_time_from_full_string_with_milliseconds() {
    let date_time = DateTime::parse("2026-05-08 12:34:56.789").unwrap();

    assert_eq!(date_time.year(), 2026);
    assert_eq!(date_time.month(), 5);
    assert_eq!(date_time.day(), 8);
    assert_eq!(date_time.hour(), 12);
    assert_eq!(date_time.minute(), 34);
    assert_eq!(date_time.second(), 56);
    assert_eq!(date_time.millisecond(), 789);
}

#[test]
fn creates_date_time_from_full_string_without_milliseconds() {
    let date_time = DateTime::parse("2026-05-08 12:34:56").unwrap();

    assert_eq!(date_time.to_string(), "2026-05-08 12:34:56.000");
    assert_eq!(date_time.millisecond(), 0);
}

#[test]
fn creates_date_time_from_date_and_hour_minute_string() {
    let date_time = DateTime::parse("2026-05-08 12:34").unwrap();

    assert_eq!(date_time.to_string(), "2026-05-08 12:34:00.000");
    assert_eq!(date_time.second(), 0);
    assert_eq!(date_time.millisecond(), 0);
}

#[test]
fn creates_date_time_from_date_string() {
    let date_time = DateTime::parse("2026-05-08").unwrap();

    assert_eq!(date_time.to_string(), "2026-05-08 00:00:00.000");
    assert_eq!(date_time.hour(), 0);
    assert_eq!(date_time.minute(), 0);
    assert_eq!(date_time.second(), 0);
    assert_eq!(date_time.millisecond(), 0);
}

#[test]
fn creates_date_time_from_time_strings_using_unix_epoch_date() {
    assert_eq!(
        DateTime::parse("12:34:56.789").unwrap().to_string(),
        "1970-01-01 12:34:56.789"
    );
    assert_eq!(
        DateTime::parse("12:34:56").unwrap().to_string(),
        "1970-01-01 12:34:56.000"
    );
    assert_eq!(
        DateTime::parse("12:34").unwrap().to_string(),
        "1970-01-01 12:34:00.000"
    );
}

#[test]
fn creates_date_time_with_from_str() {
    let date_time = DateTime::from_str("2026-05-08 12:34:56.789").unwrap();

    assert_eq!(date_time.to_string(), "2026-05-08 12:34:56.789");
}

#[test]
fn narrows_date_time_to_date_and_time() {
    let date_time = DateTime::parse("2026-05-08 12:34:56.789").unwrap();
    let date = date_time.date();
    let time = date_time.time();

    assert_eq!(date.to_string(), "2026-05-08");
    assert_eq!(time.to_string(), "12:34:56.789");
}

#[test]
fn formats_date_time_with_milliseconds_even_when_zero() {
    let date_time = DateTime::parse("2026-05-08 12:34:56").unwrap();

    assert_eq!(date_time.to_string(), "2026-05-08 12:34:56.000");
}

#[test]
fn formats_date_time_with_milliseconds() {
    let date_time = DateTime::parse("2026-05-08 12:34:56.007").unwrap();

    assert_eq!(date_time.to_string(), "2026-05-08 12:34:56.007");
}

#[test]
fn formats_date_time_to_seconds() {
    let date_time = DateTime::parse("2026-05-08 12:34:56.789").unwrap();

    assert_eq!(date_time.to_string_secs(), "2026-05-08 12:34:56");
}

#[test]
fn formats_date_time_to_minutes() {
    let date_time = DateTime::parse("2026-05-08 12:34:56.789").unwrap();

    assert_eq!(date_time.to_string_mins(), "2026-05-08 12:34");
}

#[test]
fn adds_duration_to_date_time() {
    let date_time = DateTime::parse("2026-05-08 23:59:59.900").unwrap();
    let result = date_time.add_duration(Duration::from_millis(200));

    assert_eq!(result.to_string(), "2026-05-09 00:00:00.100");
}

#[test]
fn adds_negative_duration_to_date_time() {
    let date_time = DateTime::parse("2026-05-08 00:00:00.100").unwrap();
    let result = date_time.add_duration(Duration::from_millis(-200));

    assert_eq!(result.to_string(), "2026-05-07 23:59:59.900");
}

#[test]
fn subtracts_duration_from_date_time() {
    let date_time = DateTime::parse("2026-05-08 00:00:00.100").unwrap();
    let result = date_time.sub_duration(Duration::from_millis(200));

    assert_eq!(result.to_string(), "2026-05-07 23:59:59.900");
}

#[test]
fn subtracts_date_time_as_duration() {
    let left = DateTime::parse("2026-05-09 00:00:00.100").unwrap();
    let right = DateTime::parse("2026-05-08 23:59:59.900").unwrap();

    assert_eq!((left - right).as_millis(), 200);
    assert_eq!((right - left).as_millis(), -200);
}

#[test]
fn rejects_inconsistent_fields() {
    assert!(DateTime::new(2025, 2, 29, 0, 0, 0, 0).is_err());
    assert!(DateTime::new(2024, 2, 29, 24, 0, 0, 0).is_err());
    assert!(DateTime::new(2024, 2, 29, 23, 60, 0, 0).is_err());
    assert!(DateTime::new(2024, 2, 29, 23, 59, 60, 0).is_err());
    assert!(DateTime::new(2024, 2, 29, 23, 59, 59, 1_000).is_err());
}

#[test]
fn rejects_unsupported_string_formats() {
    assert!(DateTime::parse("2026/05/08").is_err());
    assert!(DateTime::parse("2026-05-08T12:34:56").is_err());
    assert!(DateTime::parse("12:34:56.78").is_err());
    assert!(DateTime::parse("12:34:56.7890").is_err());
}

#[test]
fn now_creates_valid_date_time() {
    let date_time = DateTime::now();

    assert!(date_time.month() >= 1 && date_time.month() <= 12);
    assert!(date_time.day() >= 1 && date_time.day() <= 31);
    assert!(date_time.hour() <= 23);
    assert!(date_time.minute() <= 59);
    assert!(date_time.second() <= 59);
    assert!(date_time.millisecond() <= 999);
}
