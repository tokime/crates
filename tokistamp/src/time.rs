use std::fmt;
use std::ops::Sub;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{DateTime, Duration, ParseDateTimeError};

/// UTC time of day with millisecond precision and no timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
}

impl Time {
    #[inline(always)]
    pub(crate) const fn from_validated(hour: u8, minute: u8, second: u8, millisecond: u16) -> Self {
        Self {
            hour,
            minute,
            second,
            millisecond,
        }
    }

    /// Returns the current UTC time from `SystemTime::now()`.
    #[inline(always)]
    pub fn now() -> Self {
        DateTime::now().time()
    }

    /// Parses a time from a string through `DateTime` and narrows it to `Time`.
    ///
    /// Accepts `HH:mm:ss.SSS`, `HH:mm:ss`, `HH:mm`,
    /// `yyyy-MM-dd HH:mm:ss.SSS`, `yyyy-MM-dd HH:mm:ss`, and
    /// `yyyy-MM-dd HH:mm`.
    ///
    /// Returns an error when the string format is unsupported or date/time
    /// fields are out of range.
    #[inline(always)]
    pub fn parse(value: &str) -> Result<Self, ParseDateTimeError> {
        DateTime::parse(value).map(|date_time| date_time.time())
    }

    /// Adds a duration from `1970-01-01` with this time and returns `DateTime`.
    #[inline(always)]
    pub fn add_duration(self, duration: Duration) -> DateTime {
        DateTime::new(
            1970,
            1,
            1,
            self.hour,
            self.minute,
            self.second,
            self.millisecond,
        )
        .expect("time fields must be valid")
        .add_duration(duration)
    }

    /// Subtracts a duration from `1970-01-01` with this time and returns `DateTime`.
    #[inline(always)]
    pub fn sub_duration(self, duration: Duration) -> DateTime {
        DateTime::new(
            1970,
            1,
            1,
            self.hour,
            self.minute,
            self.second,
            self.millisecond,
        )
        .expect("time fields must be valid")
        .sub_duration(duration)
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

    /// Formats as `HH:mm:ss`.
    #[inline(always)]
    pub fn to_string_secs(self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }

    /// Formats as `HH:mm`.
    #[inline(always)]
    pub fn to_string_mins(self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    #[inline(always)]
    fn as_day_millis(self) -> i64 {
        self.hour as i64 * 3_600_000
            + self.minute as i64 * 60_000
            + self.second as i64 * 1_000
            + self.millisecond as i64
    }
}

impl Sub for Time {
    type Output = Duration;

    /// Returns the millisecond difference between two time values.
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Duration::from_millis(self.as_day_millis() - rhs.as_day_millis())
    }
}

impl fmt::Display for Time {
    /// Formats as `HH:mm:ss.SSS`.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:02}:{:02}:{:02}.{:03}",
            self.hour, self.minute, self.second, self.millisecond
        )
    }
}

impl FromStr for Time {
    type Err = ParseDateTimeError;

    /// Parses a time from the same formats as `Time::parse`.
    #[inline(always)]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            serializer.serialize_i64(self.as_day_millis())
        }
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let value = String::deserialize(deserializer)?;
            Self::parse(&value).map_err(serde::de::Error::custom)
        } else {
            let milliseconds = i64::deserialize(deserializer)?;
            if !(0..86_400_000).contains(&milliseconds) {
                return Err(serde::de::Error::custom(
                    "time milliseconds must be in 0..86400000",
                ));
            }

            let hour = (milliseconds / 3_600_000) as u8;
            let minute = (milliseconds % 3_600_000 / 60_000) as u8;
            let second = (milliseconds % 60_000 / 1_000) as u8;
            let millisecond = (milliseconds % 1_000) as u16;

            Ok(Self::from_validated(hour, minute, second, millisecond))
        }
    }
}
