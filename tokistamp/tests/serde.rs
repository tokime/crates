use tokistamp::{Date, DateTime, Datestamp, Time, Timestamp};

#[test]
fn human_readable_serializes_as_strings() {
    let timestamp = Timestamp::from_millis(86_400_123);
    let datestamp = Datestamp::from_days(2);
    let date = Date::parse("2026-05-08").unwrap();
    let time = Time::parse("12:34:56.789").unwrap();
    let date_time = DateTime::parse("2026-05-08 12:34:56.789").unwrap();

    assert_eq!(
        serde_json::to_string(&timestamp).unwrap(),
        "\"1970-01-02 00:00:00.123\""
    );
    assert_eq!(serde_json::to_string(&datestamp).unwrap(), "\"1970-01-03\"");
    assert_eq!(serde_json::to_string(&date).unwrap(), "\"2026-05-08\"");
    assert_eq!(serde_json::to_string(&time).unwrap(), "\"12:34:56.789\"");
    assert_eq!(
        serde_json::to_string(&date_time).unwrap(),
        "\"2026-05-08 12:34:56.789\""
    );

    assert_eq!(
        serde_json::from_str::<Timestamp>("\"1970-01-02 00:00:00.123\"").unwrap(),
        timestamp
    );
    assert_eq!(
        serde_json::from_str::<Datestamp>("\"1970-01-03\"").unwrap(),
        datestamp
    );
    assert_eq!(
        serde_json::from_str::<Date>("\"2026-05-08\"").unwrap(),
        date
    );
    assert_eq!(
        serde_json::from_str::<Time>("\"12:34:56.789\"").unwrap(),
        time
    );
    assert_eq!(
        serde_json::from_str::<DateTime>("\"2026-05-08 12:34:56.789\"").unwrap(),
        date_time
    );
}

#[test]
fn postcard_serializes_as_numbers() {
    let timestamp = Timestamp::from_millis(86_400_123);
    let datestamp = Datestamp::from_days(2);
    let date = Date::parse("1970-01-03").unwrap();
    let time = Time::parse("12:34:56.789").unwrap();
    let date_time = DateTime::parse("2026-05-08 12:34:56.789").unwrap();

    let date_time_millis = Timestamp::from(date_time).as_i64();

    assert_eq!(
        postcard::to_allocvec(&timestamp).unwrap(),
        postcard::to_allocvec(&86_400_123_i64).unwrap()
    );
    assert_eq!(
        postcard::to_allocvec(&datestamp).unwrap(),
        postcard::to_allocvec(&2_i32).unwrap()
    );
    assert_eq!(
        postcard::to_allocvec(&date).unwrap(),
        postcard::to_allocvec(&2_i32).unwrap()
    );
    assert_eq!(
        postcard::to_allocvec(&time).unwrap(),
        postcard::to_allocvec(&45_296_789_i64).unwrap()
    );
    assert_eq!(
        postcard::to_allocvec(&date_time).unwrap(),
        postcard::to_allocvec(&date_time_millis).unwrap()
    );

    assert_eq!(
        postcard::from_bytes::<Timestamp>(&postcard::to_allocvec(&86_400_123_i64).unwrap())
            .unwrap(),
        timestamp
    );
    assert_eq!(
        postcard::from_bytes::<Datestamp>(&postcard::to_allocvec(&2_i32).unwrap()).unwrap(),
        datestamp
    );
    assert_eq!(
        postcard::from_bytes::<Date>(&postcard::to_allocvec(&2_i32).unwrap()).unwrap(),
        date
    );
    assert_eq!(
        postcard::from_bytes::<Time>(&postcard::to_allocvec(&45_296_789_i64).unwrap()).unwrap(),
        time
    );
    assert_eq!(
        postcard::from_bytes::<DateTime>(&postcard::to_allocvec(&date_time_millis).unwrap())
            .unwrap(),
        date_time
    );
}

#[test]
fn postcard_rejects_out_of_range_time_millis() {
    let bytes = postcard::to_allocvec(&86_400_000_i64).unwrap();

    assert!(postcard::from_bytes::<Time>(&bytes).is_err());
}
