use std::fmt;
use std::ops::Sub;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{DateTime, Datestamp, Duration, ParseDateTimeError};

/// UTC calendar date without a timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    #[inline(always)]
    pub(crate) const fn from_validated(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Returns the current UTC date from `SystemTime::now()`.
    #[inline(always)]
    pub fn now() -> Self {
        DateTime::now().date()
    }

    /// Parses a date from a string through `DateTime` and narrows it to `Date`.
    ///
    /// Accepts `yyyy-MM-dd`, `yyyy-MM-dd HH:mm:ss.SSS`,
    /// `yyyy-MM-dd HH:mm:ss`, and `yyyy-MM-dd HH:mm`.
    ///
    /// Returns an error when the string format is unsupported or date/time
    /// fields are out of range.
    #[inline(always)]
    pub fn parse(value: &str) -> Result<Self, ParseDateTimeError> {
        DateTime::parse(value).map(|date_time| date_time.date())
    }

    /// Adds a duration from midnight of this date and returns `DateTime`.
    #[inline(always)]
    pub fn add_duration(self, duration: Duration) -> DateTime {
        DateTime::new(self.year, self.month, self.day, 0, 0, 0, 0)
            .expect("date fields must be valid")
            .add_duration(duration)
    }

    /// Subtracts a duration from midnight of this date and returns `DateTime`.
    #[inline(always)]
    pub fn sub_duration(self, duration: Duration) -> DateTime {
        DateTime::new(self.year, self.month, self.day, 0, 0, 0, 0)
            .expect("date fields must be valid")
            .sub_duration(duration)
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
}

impl Sub for Date {
    type Output = Duration;

    /// Returns the millisecond difference between two dates at midnight.
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        let left = DateTime::new(self.year, self.month, self.day, 0, 0, 0, 0)
            .expect("date fields must be valid");
        let right = DateTime::new(rhs.year, rhs.month, rhs.day, 0, 0, 0, 0)
            .expect("date fields must be valid");

        left - right
    }
}

impl fmt::Display for Date {
    /// Formats the date as `yyyy-MM-dd`.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

impl FromStr for Date {
    type Err = ParseDateTimeError;

    /// Parses a date from the same formats as `Date::parse`.
    #[inline(always)]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl From<Datestamp> for Date {
    /// Converts Unix epoch days to `Date`.
    #[inline(always)]
    fn from(datestamp: Datestamp) -> Self {
        DateTime::from_unix_millis(datestamp.as_i32() as i64 * 86_400_000).date()
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            serializer.serialize_i32(Datestamp::from(*self).as_i32())
        }
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let value = String::deserialize(deserializer)?;
            Self::parse(&value).map_err(serde::de::Error::custom)
        } else {
            i32::deserialize(deserializer)
                .map(Datestamp::from)
                .map(Self::from)
        }
    }
}
