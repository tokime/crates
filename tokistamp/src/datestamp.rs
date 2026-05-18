use std::fmt;
use std::ops::{Add, Sub};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Date, DateTime, Duration, Timestamp};

const MILLIS_PER_DAY: i64 = 86_400_000;

/// Days since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Datestamp(i32);

impl Datestamp {
    /// Creates a datestamp from Unix epoch days.
    #[inline(always)]
    pub fn from_days(days: i32) -> Self {
        Self(days)
    }

    /// Returns the current Unix epoch days from `SystemTime::now()`.
    #[inline(always)]
    pub fn now() -> Self {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must not be before Unix epoch");
        let days = (elapsed.as_millis() / MILLIS_PER_DAY as u128) as i32;

        Self(days)
    }

    /// Returns the datestamp as Unix epoch days.
    #[inline(always)]
    pub fn as_i32(self) -> i32 {
        self.0
    }

    /// Adds a duration from midnight of this datestamp and returns `Timestamp`.
    #[inline(always)]
    pub fn add_duration(self, duration: Duration) -> Timestamp {
        Timestamp::from_millis(self.as_unix_millis() + duration.as_millis())
    }

    /// Subtracts a duration from midnight of this datestamp and returns `Timestamp`.
    #[inline(always)]
    pub fn sub_duration(self, duration: Duration) -> Timestamp {
        Timestamp::from_millis(self.as_unix_millis() - duration.as_millis())
    }

    #[inline(always)]
    fn as_unix_millis(self) -> i64 {
        self.0 as i64 * MILLIS_PER_DAY
    }
}

impl From<i32> for Datestamp {
    /// Creates a datestamp from Unix epoch days.
    #[inline(always)]
    fn from(days: i32) -> Self {
        Self::from_days(days)
    }
}

impl From<DateTime> for Datestamp {
    /// Converts `DateTime` to Unix epoch days.
    #[inline(always)]
    fn from(date_time: DateTime) -> Self {
        let days = date_time.as_unix_millis().div_euclid(MILLIS_PER_DAY);
        Self(days.try_into().expect("Unix epoch days must fit into i32"))
    }
}

impl From<Date> for Datestamp {
    /// Converts `Date` to Unix epoch days.
    #[inline(always)]
    fn from(date: Date) -> Self {
        DateTime::new(date.year(), date.month(), date.day(), 0, 0, 0, 0)
            .expect("date fields must be valid")
            .into()
    }
}

impl Add<Duration> for Datestamp {
    type Output = Timestamp;

    /// Adds a duration from midnight of this datestamp.
    #[inline(always)]
    fn add(self, rhs: Duration) -> Self::Output {
        self.add_duration(rhs)
    }
}

impl Sub<Duration> for Datestamp {
    type Output = Timestamp;

    /// Subtracts a duration from midnight of this datestamp.
    #[inline(always)]
    fn sub(self, rhs: Duration) -> Self::Output {
        self.sub_duration(rhs)
    }
}

impl Sub for Datestamp {
    type Output = Duration;

    /// Returns the millisecond difference between two datestamps at midnight.
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Duration::from_days((self.0 - rhs.0) as i64)
    }
}

impl fmt::Display for Datestamp {
    /// Formats as `yyyy-MM-dd`.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        DateTime::from_unix_millis(self.as_unix_millis())
            .date()
            .fmt(formatter)
    }
}

impl Serialize for Datestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            serializer.serialize_i32(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Datestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let value = String::deserialize(deserializer)?;
            Date::parse(&value)
                .map(Self::from)
                .map_err(serde::de::Error::custom)
        } else {
            i32::deserialize(deserializer).map(Self)
        }
    }
}
