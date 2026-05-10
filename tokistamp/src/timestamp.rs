use std::fmt;
use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

use crate::{DateTime, Duration};

/// Milliseconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Creates a timestamp from Unix epoch milliseconds.
    #[inline(always)]
    pub fn from_millis(milliseconds: i64) -> Self {
        Self(milliseconds)
    }

    /// Returns the timestamp as Unix epoch milliseconds.
    #[inline(always)]
    pub fn as_i64(self) -> i64 {
        self.0
    }

    /// Adds a duration and returns the resulting `Timestamp`.
    #[inline(always)]
    pub fn add_duration(self, duration: Duration) -> Self {
        Self(self.0 + duration.as_millis())
    }

    /// Subtracts a duration and returns the resulting `Timestamp`.
    #[inline(always)]
    pub fn sub_duration(self, duration: Duration) -> Self {
        Self(self.0 - duration.as_millis())
    }
}

impl From<i64> for Timestamp {
    /// Creates a timestamp from Unix epoch milliseconds.
    #[inline(always)]
    fn from(milliseconds: i64) -> Self {
        Self::from_millis(milliseconds)
    }
}

impl From<DateTime> for Timestamp {
    /// Converts `DateTime` to Unix epoch milliseconds.
    #[inline(always)]
    fn from(date_time: DateTime) -> Self {
        Self(date_time.as_unix_millis())
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;

    /// Adds a duration to a timestamp.
    #[inline(always)]
    fn add(self, rhs: Duration) -> Self::Output {
        self.add_duration(rhs)
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Self;

    /// Subtracts a duration from a timestamp.
    #[inline(always)]
    fn sub(self, rhs: Duration) -> Self::Output {
        self.sub_duration(rhs)
    }
}

impl Sub for Timestamp {
    type Output = Duration;

    /// Returns the millisecond difference between two timestamps.
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Duration::from_millis(self.0 - rhs.0)
    }
}

impl fmt::Display for Timestamp {
    /// Formats as `yyyy-MM-dd HH:mm:ss.SSS`.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        DateTime::from_unix_millis(self.0).fmt(formatter)
    }
}
