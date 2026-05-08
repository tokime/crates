use tokistamp::Duration;

#[test]
fn creates_duration_from_milliseconds() {
    let duration = Duration::from_millis(123);

    assert_eq!(duration.as_millis(), 123);
}

#[test]
fn creates_duration_from_seconds() {
    let duration = Duration::from_secs(2);

    assert_eq!(duration.as_millis(), 2_000);
}

#[test]
fn creates_duration_from_minutes() {
    let duration = Duration::from_mins(2);

    assert_eq!(duration.as_millis(), 120_000);
}

#[test]
fn creates_duration_from_hours() {
    let duration = Duration::from_hours(2);

    assert_eq!(duration.as_millis(), 7_200_000);
}

#[test]
fn creates_duration_from_days() {
    let duration = Duration::from_days(2);

    assert_eq!(duration.as_millis(), 172_800_000);
}

#[test]
fn creates_negative_duration() {
    let duration = Duration::from_secs(-2);

    assert_eq!(duration.as_millis(), -2_000);
}

#[test]
fn formats_duration_as_milliseconds() {
    let duration = Duration::from_millis(123);

    assert_eq!(duration.to_string(), "123ms");
}

#[test]
fn formats_duration_as_non_zero_parts() {
    let duration = Duration::from_days(12)
        + Duration::from_hours(23)
        + Duration::from_mins(45)
        + Duration::from_secs(56)
        + Duration::from_millis(123);

    assert_eq!(duration.to_string(), "12d 23h 45m 56s 123ms");
}

#[test]
fn formats_duration_without_zero_parts() {
    let duration = Duration::from_days(1) + Duration::from_secs(2);

    assert_eq!(duration.to_string(), "1d 2s");
}

#[test]
fn formats_negative_duration_as_non_zero_parts() {
    let duration = Duration::from_millis(-123);

    assert_eq!(duration.to_string(), "-123ms");
}

#[test]
fn formats_zero_duration_as_zero_milliseconds() {
    let duration = Duration::from_millis(0);

    assert_eq!(duration.to_string(), "0ms");
}
