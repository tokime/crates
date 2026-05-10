use std::error::Error;
use std::fmt;
use std::ops::Sub;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{Date, Duration, Time};

const DEFAULT_YEAR: i32 = 1970;
const DEFAULT_MONTH: u8 = 1;
const DEFAULT_DAY: u8 = 1;

/// UTC date and time with millisecond precision and no timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DateTime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
}

/// Error returned when date/time parsing or validation fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDateTimeError {
    message: &'static str,
}

impl ParseDateTimeError {
    #[inline(always)]
    const fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl fmt::Display for ParseDateTimeError {
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message)
    }
}

impl Error for ParseDateTimeError {}

impl DateTime {
    /// Creates a `DateTime` from checked date and time fields.
    ///
    /// Returns an error when `month` is not in `1..=12`, `day` is invalid for
    /// the month, `hour` is not in `0..=23`, `minute` or `second` is not in
    /// `0..=59`, or `millisecond` is not in `0..=999`.
    #[inline(always)]
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
    ) -> Result<Self, ParseDateTimeError> {
        validate_date(year, month, day)?;
        validate_time(hour, minute, second, millisecond)?;

        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
        })
    }

    /// Returns the current UTC date and time from `SystemTime::now()`.
    #[inline(always)]
    pub fn now() -> Self {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must not be before Unix epoch");
        let milliseconds = elapsed
            .as_millis()
            .try_into()
            .expect("current Unix milliseconds must fit into i64");

        Self::from_unix_millis(milliseconds)
    }

    /// Parses a `DateTime` from a supported string format.
    ///
    /// Accepts `yyyy-MM-dd'T'HH:mm:ss.SSSZ`,
    /// `yyyy-MM-dd'T'HH:mm:ssZ`, `yyyy-MM-dd HH:mm:ss.SSS`,
    /// `yyyy-MM-dd HH:mm:ss`, `yyyy-MM-dd HH:mm`, `yyyy-MM-dd`,
    /// `HH:mm:ss.SSS`, `HH:mm:ss`, and `HH:mm`.
    ///
    /// Date-only strings use `00:00:00.000`. Time-only strings use
    /// `1970-01-01` as the date.
    ///
    /// Returns an error when the string format is unsupported, contains
    /// non-ASCII characters, has invalid separators, or fields are out of
    /// range.
    #[inline(always)]
    pub fn parse(value: &str) -> Result<Self, ParseDateTimeError> {
        if let Some((date, time)) = parse_rfc3339_utc(value)? {
            let (year, month, day) = parse_date(date)?;
            let (hour, minute, second, millisecond) = parse_time(time)?;
            return Self::new(year, month, day, hour, minute, second, millisecond);
        }

        if let Some((date, time)) = value.split_once(' ') {
            let (year, month, day) = parse_date(date)?;
            let (hour, minute, second, millisecond) = parse_time(time)?;
            return Self::new(year, month, day, hour, minute, second, millisecond);
        }

        if value.contains('-') {
            let (year, month, day) = parse_date(value)?;
            return Self::new(year, month, day, 0, 0, 0, 0);
        }

        if value.contains(':') {
            let (hour, minute, second, millisecond) = parse_time(value)?;
            return Self::new(
                DEFAULT_YEAR,
                DEFAULT_MONTH,
                DEFAULT_DAY,
                hour,
                minute,
                second,
                millisecond,
            );
        }

        Err(ParseDateTimeError::new("unsupported date time format"))
    }

    /// Returns the date part.
    #[inline(always)]
    pub fn date(self) -> Date {
        Date::from_validated(self.year, self.month, self.day)
    }

    /// Returns the time part.
    #[inline(always)]
    pub fn time(self) -> Time {
        Time::from_validated(self.hour, self.minute, self.second, self.millisecond)
    }

    /// Adds a duration and returns the resulting `DateTime`.
    #[inline(always)]
    pub fn add_duration(self, duration: Duration) -> Self {
        Self::from_unix_millis(self.as_unix_millis() + duration.as_millis())
    }

    /// Subtracts a duration and returns the resulting `DateTime`.
    #[inline(always)]
    pub fn sub_duration(self, duration: Duration) -> Self {
        Self::from_unix_millis(self.as_unix_millis() - duration.as_millis())
    }

    /// Returns the year.
    #[inline(always)]
    pub fn year(self) -> i32 {
        self.year
    }

    /// Returns the month in `1..=12`.
    #[inline(always)]
    pub fn month(self) -> u8 {
        self.month
    }

    /// Returns the day of month.
    #[inline(always)]
    pub fn day(self) -> u8 {
        self.day
    }

    /// Returns the hour in `0..=23`.
    #[inline(always)]
    pub fn hour(self) -> u8 {
        self.hour
    }

    /// Returns the minute in `0..=59`.
    #[inline(always)]
    pub fn minute(self) -> u8 {
        self.minute
    }

    /// Returns the second in `0..=59`.
    #[inline(always)]
    pub fn second(self) -> u8 {
        self.second
    }

    /// Returns the millisecond in `0..=999`.
    #[inline(always)]
    pub fn millisecond(self) -> u16 {
        self.millisecond
    }

    /// Formats as `yyyy-MM-dd HH:mm:ss`.
    #[inline(always)]
    pub fn to_string_secs(self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }

    /// Formats as `yyyy-MM-dd HH:mm`.
    #[inline(always)]
    pub fn to_string_mins(self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }

    #[inline(always)]
    pub fn from_unix_millis(milliseconds: i64) -> Self {
        let days = milliseconds.div_euclid(86_400_000);
        let day_millis = milliseconds.rem_euclid(86_400_000);
        let (year, month, day) = civil_from_days(days);

        let hour = (day_millis / 3_600_000) as u8;
        let minute = (day_millis % 3_600_000 / 60_000) as u8;
        let second = (day_millis % 60_000 / 1_000) as u8;
        let millisecond = (day_millis % 1_000) as u16;

        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
        }
    }

    #[inline(always)]
    pub(crate) fn as_unix_millis(self) -> i64 {
        let days = days_from_civil(self.year, self.month, self.day);
        let day_millis = self.hour as i64 * 3_600_000
            + self.minute as i64 * 60_000
            + self.second as i64 * 1_000
            + self.millisecond as i64;

        days * 86_400_000 + day_millis
    }
}

impl Sub for DateTime {
    type Output = Duration;

    /// Returns the millisecond difference between two `DateTime` values.
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Duration::from_millis(self.as_unix_millis() - rhs.as_unix_millis())
    }
}

impl fmt::Display for DateTime {
    /// Formats as `yyyy-MM-dd HH:mm:ss.SSS`.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            self.year, self.month, self.day, self.hour, self.minute, self.second, self.millisecond
        )
    }
}

impl FromStr for DateTime {
    type Err = ParseDateTimeError;

    /// Parses a `DateTime` from the same formats as `DateTime::parse`.
    #[inline(always)]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[inline(always)]
fn parse_rfc3339_utc(value: &str) -> Result<Option<(&str, &str)>, ParseDateTimeError> {
    let Some(without_z) = value.strip_suffix('Z') else {
        return Ok(None);
    };
    let Some((date, time)) = without_z.split_once('T') else {
        return Ok(None);
    };
    if date.len() != 10 {
        return Err(ParseDateTimeError::new(
            "RFC 3339 date time must use yyyy-MM-ddTHH:mm:ssZ or yyyy-MM-ddTHH:mm:ss.SSSZ format",
        ));
    }
    match time.len() {
        8 => Ok(Some((date, time))),
        12 if time.as_bytes()[8] == b'.' => Ok(Some((date, time))),
        _ => Err(ParseDateTimeError::new(
            "RFC 3339 date time must use yyyy-MM-ddTHH:mm:ssZ or yyyy-MM-ddTHH:mm:ss.SSSZ format",
        )),
    }
}

#[inline(always)]
fn parse_date(value: &str) -> Result<(i32, u8, u8), ParseDateTimeError> {
    if !value.is_ascii() {
        return Err(ParseDateTimeError::new(
            "date must contain ASCII characters",
        ));
    }

    if value.len() != 10 {
        return Err(ParseDateTimeError::new("date must use yyyy-MM-dd format"));
    }

    let year = parse_i32(&value[0..4], "year")?;
    expect_byte(value, 4, b'-')?;
    let month = parse_u8(&value[5..7], "month")?;
    expect_byte(value, 7, b'-')?;
    let day = parse_u8(&value[8..10], "day")?;

    validate_date(year, month, day)?;
    Ok((year, month, day))
}

#[inline(always)]
fn parse_time(value: &str) -> Result<(u8, u8, u8, u16), ParseDateTimeError> {
    if !value.is_ascii() {
        return Err(ParseDateTimeError::new(
            "time must contain ASCII characters",
        ));
    }

    match value.len() {
        5 => {
            let hour = parse_u8(&value[0..2], "hour")?;
            expect_byte(value, 2, b':')?;
            let minute = parse_u8(&value[3..5], "minute")?;
            validate_time(hour, minute, 0, 0)?;
            Ok((hour, minute, 0, 0))
        }
        8 => {
            let hour = parse_u8(&value[0..2], "hour")?;
            expect_byte(value, 2, b':')?;
            let minute = parse_u8(&value[3..5], "minute")?;
            expect_byte(value, 5, b':')?;
            let second = parse_u8(&value[6..8], "second")?;
            validate_time(hour, minute, second, 0)?;
            Ok((hour, minute, second, 0))
        }
        12 => {
            let hour = parse_u8(&value[0..2], "hour")?;
            expect_byte(value, 2, b':')?;
            let minute = parse_u8(&value[3..5], "minute")?;
            expect_byte(value, 5, b':')?;
            let second = parse_u8(&value[6..8], "second")?;
            expect_byte(value, 8, b'.')?;
            let millisecond = parse_u16(&value[9..12], "millisecond")?;
            validate_time(hour, minute, second, millisecond)?;
            Ok((hour, minute, second, millisecond))
        }
        _ => Err(ParseDateTimeError::new(
            "time must use HH:mm, HH:mm:ss, or HH:mm:ss.SSS format",
        )),
    }
}

#[inline(always)]
fn validate_date(year: i32, month: u8, day: u8) -> Result<(), ParseDateTimeError> {
    if month == 0 || month > 12 {
        return Err(ParseDateTimeError::new("month must be in 1..=12"));
    }

    let max_day = days_in_month(year, month);
    if day == 0 || day > max_day {
        return Err(ParseDateTimeError::new("day is out of range for month"));
    }

    Ok(())
}

#[inline(always)]
fn validate_time(
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
) -> Result<(), ParseDateTimeError> {
    if hour > 23 {
        return Err(ParseDateTimeError::new("hour must be in 0..=23"));
    }
    if minute > 59 {
        return Err(ParseDateTimeError::new("minute must be in 0..=59"));
    }
    if second > 59 {
        return Err(ParseDateTimeError::new("second must be in 0..=59"));
    }
    if millisecond > 999 {
        return Err(ParseDateTimeError::new("millisecond must be in 0..=999"));
    }

    Ok(())
}

#[inline(always)]
fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

#[inline(always)]
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[inline(always)]
fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };

    (year as i32, month as u8, day as u8)
}

#[inline(always)]
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let mut year = year as i64;
    let month = month as i64;
    let day = day as i64;

    if month <= 2 {
        year -= 1;
    }

    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

#[inline(always)]
fn expect_byte(value: &str, index: usize, expected: u8) -> Result<(), ParseDateTimeError> {
    match value.as_bytes().get(index) {
        Some(actual) if *actual == expected => Ok(()),
        _ => Err(ParseDateTimeError::new("date time separator is invalid")),
    }
}

#[inline(always)]
fn parse_i32(value: &str, field: &'static str) -> Result<i32, ParseDateTimeError> {
    if !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return Err(ParseDateTimeError::new(field_error(field)));
    }

    value
        .parse()
        .map_err(|_| ParseDateTimeError::new(field_error(field)))
}

#[inline(always)]
fn parse_u8(value: &str, field: &'static str) -> Result<u8, ParseDateTimeError> {
    if !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return Err(ParseDateTimeError::new(field_error(field)));
    }

    value
        .parse()
        .map_err(|_| ParseDateTimeError::new(field_error(field)))
}

#[inline(always)]
fn parse_u16(value: &str, field: &'static str) -> Result<u16, ParseDateTimeError> {
    if !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return Err(ParseDateTimeError::new(field_error(field)));
    }

    value
        .parse()
        .map_err(|_| ParseDateTimeError::new(field_error(field)))
}

#[inline(always)]
fn field_error(field: &str) -> &'static str {
    match field {
        "year" => "year must contain digits",
        "month" => "month must contain digits",
        "day" => "day must contain digits",
        "hour" => "hour must contain digits",
        "minute" => "minute must contain digits",
        "second" => "second must contain digits",
        "millisecond" => "millisecond must contain digits",
        _ => "date time field must contain digits",
    }
}
