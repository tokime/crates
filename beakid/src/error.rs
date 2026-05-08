//! Error types returned by BeakId.

use std::error::Error;
use std::fmt;

/// Crate-local result type.
pub type Result<T> = std::result::Result<T, BeakIdError>;

/// Errors returned by BeakId configuration and generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeakIdError {
    /// `BEAKID_EPOCH` was not present.
    MissingEpoch,
    /// `BEAKID_EPOCH` was not a valid RFC 3339 UTC timestamp.
    InvalidEpoch(String),
    /// `BEAKID_WORKER_ID` was not a valid unsigned integer.
    InvalidWorkerId(String),
    /// `BEAKID_WORKER_ID` exceeded the 10-bit worker-id range.
    WorkerIdOutOfRange(u16),
    /// The system clock is before the configured epoch.
    ClockBeforeEpoch,
    /// The timestamp exceeded the 35-bit BeakId timestamp range.
    TimestampOverflow(u64),
    /// The standard background thread could not be spawned.
    BackgroundSpawnFailed(String),
}

impl fmt::Display for BeakIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEpoch => f.write_str("BEAKID_EPOCH is required"),
            Self::InvalidEpoch(value) => write!(
                f,
                "BEAKID_EPOCH must be an RFC 3339 UTC datetime, got {value:?}"
            ),
            Self::InvalidWorkerId(value) => {
                write!(
                    f,
                    "BEAKID_WORKER_ID must be an unsigned 16-bit integer, got {value:?}"
                )
            }
            Self::WorkerIdOutOfRange(value) => {
                write!(f, "BEAKID_WORKER_ID must fit in 10 bits, got {value}")
            }
            Self::ClockBeforeEpoch => f.write_str("system clock is before BEAKID_EPOCH"),
            Self::TimestampOverflow(value) => {
                write!(f, "timestamp window {value} exceeds the 35-bit range")
            }
            Self::BackgroundSpawnFailed(value) => {
                write!(f, "failed to spawn BeakId background thread: {value}")
            }
        }
    }
}

impl Error for BeakIdError {}
