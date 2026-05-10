use std::fmt;
use std::ops::Sub;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{DateTime, Duration, ParseDateTimeError};

/// UTC calendar date without a timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
