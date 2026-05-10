use std::fmt;
use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

/// Duration stored as milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Duration(i64);

impl Duration {
    /// Creates a duration from milliseconds.
    #[inline(always)]
    pub fn from_millis(milliseconds: i64) -> Self {
        Self(milliseconds)
    }

    /// Creates a duration from seconds.
    #[inline(always)]
    pub fn from_secs(seconds: i64) -> Self {
        Self(seconds * 1_000)
    }

    /// Creates a duration from minutes.
    #[inline(always)]
    pub fn from_mins(minutes: i64) -> Self {
        Self(minutes * 60_000)
    }

    /// Creates a duration from hours.
    #[inline(always)]
    pub fn from_hours(hours: i64) -> Self {
        Self(hours * 3_600_000)
    }

    /// Creates a duration from days.
    #[inline(always)]
    pub fn from_days(days: i64) -> Self {
        Self(days * 86_400_000)
    }

    /// Returns the duration as milliseconds.
    #[inline(always)]
    pub fn as_millis(self) -> i64 {
        self.0
    }
}

impl Add for Duration {
    type Output = Self;

    /// Adds two durations.
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Duration {
    type Output = Self;

    /// Subtracts two durations.
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl fmt::Display for Duration {
    /// Formats as non-zero duration parts, for example `12d 23h 45m 56s 123ms`.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut milliseconds = self.0.unsigned_abs();
        let days = milliseconds / 86_400_000;
        milliseconds %= 86_400_000;
        let hours = milliseconds / 3_600_000;
        milliseconds %= 3_600_000;
        let minutes = milliseconds / 60_000;
        milliseconds %= 60_000;
        let seconds = milliseconds / 1_000;
        milliseconds %= 1_000;

        if self.0 < 0 {
            formatter.write_str("-")?;
        }

        let mut needs_space = false;
        write_part(formatter, &mut needs_space, days, "d")?;
        write_part(formatter, &mut needs_space, hours, "h")?;
        write_part(formatter, &mut needs_space, minutes, "m")?;
        write_part(formatter, &mut needs_space, seconds, "s")?;
        write_part(formatter, &mut needs_space, milliseconds, "ms")?;

        if !needs_space {
            formatter.write_str("0ms")?;
        }

        Ok(())
    }
}

#[inline(always)]
fn write_part(
    formatter: &mut fmt::Formatter<'_>,
    needs_space: &mut bool,
    value: u64,
    suffix: &str,
) -> fmt::Result {
    if value == 0 {
        return Ok(());
    }

    if *needs_space {
        formatter.write_str(" ")?;
    }

    write!(formatter, "{value}{suffix}")?;
    *needs_space = true;

    Ok(())
}
