use std::str::FromStr;

use tokistamp::{Date, Duration};

#[test]
fn creates_date_from_date_string() {
    let date = Date::parse("2026-05-08").unwrap();

    assert_eq!(date.year(), 2026);
    assert_eq!(date.month(), 5);
    assert_eq!(date.day(), 8);
}

#[test]
fn creates_date_from_date_time_string() {
    let date = Date::parse("2026-05-08 12:34:56.789").unwrap();

    assert_eq!(date.year(), 2026);
    assert_eq!(date.month(), 5);
    assert_eq!(date.day(), 8);
}

#[test]
fn creates_date_with_from_str() {
    let date = Date::from_str("2026-05-08").unwrap();

    assert_eq!(date.to_string(), "2026-05-08");
}

#[test]
fn formats_date_as_yyyy_mm_dd() {
    let date = Date::parse("2026-05-08").unwrap();

    assert_eq!(date.to_string(), "2026-05-08");
}

#[test]
fn adds_duration_to_date_from_midnight() {
    let date = Date::parse("2026-05-08").unwrap();
    let result = date.add_duration(Duration::from_hours(26));

    assert_eq!(result.to_string(), "2026-05-09 02:00:00.000");
}

#[test]
fn subtracts_duration_from_date_midnight() {
    let date = Date::parse("2026-05-08").unwrap();
    let result = date.sub_duration(Duration::from_millis(1));

    assert_eq!(result.to_string(), "2026-05-07 23:59:59.999");
}

#[test]
fn subtracts_date_as_duration() {
    let left = Date::parse("2026-05-10").unwrap();
    let right = Date::parse("2026-05-08").unwrap();

    assert_eq!(
        (left - right).as_millis(),
        Duration::from_days(2).as_millis()
    );
    assert_eq!(
        (right - left).as_millis(),
        Duration::from_days(-2).as_millis()
    );
}

#[test]
fn rejects_invalid_date_string() {
    assert!(Date::parse("2026-02-29").is_err());
    assert!(Date::parse("2026-00-08").is_err());
    assert!(Date::parse("2026-13-08").is_err());
    assert!(Date::parse("2026-05-00").is_err());
    assert!(Date::parse("2026/05/08").is_err());
}

#[test]
fn now_creates_valid_date() {
    let date = Date::now();

    assert!(date.month() >= 1 && date.month() <= 12);
    assert!(date.day() >= 1 && date.day() <= 31);
}
