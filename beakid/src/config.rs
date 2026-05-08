//! Environment and explicit configuration parsing.

use std::env;
use std::time::{Duration, UNIX_EPOCH};

use tokistamp::{DateTime, Timestamp};

use crate::error::{BeakIdError, Result};
use crate::generator::MAX_WORKER_ID;

const EPOCH_ENV: &str = "BEAKID_EPOCH";
const WORKER_ID_ENV: &str = "BEAKID_WORKER_ID";

/// Runtime configuration for a [`crate::Generator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    epoch: std::time::SystemTime,
    worker_id: u16,
}

impl Config {
    /// Builds configuration from explicit values.
    ///
    /// `epoch` is interpreted as UTC and `worker_id` must fit in 10 bits.
    pub fn new(epoch: DateTime, worker_id: u16) -> Result<Self> {
        if worker_id > MAX_WORKER_ID {
            return Err(BeakIdError::WorkerIdOutOfRange(worker_id));
        }

        let epoch_millis = Timestamp::from(epoch).as_i64();
        let epoch_millis = u64::try_from(epoch_millis)
            .map_err(|_| BeakIdError::InvalidEpoch(epoch.to_string()))?;

        Ok(Self {
            epoch: UNIX_EPOCH + Duration::from_millis(epoch_millis),
            worker_id,
        })
    }

    /// Builds configuration from a [`std::time::SystemTime`] epoch.
    pub fn from_system_time(epoch: std::time::SystemTime, worker_id: u16) -> Result<Self> {
        if worker_id > MAX_WORKER_ID {
            return Err(BeakIdError::WorkerIdOutOfRange(worker_id));
        }
        if epoch.duration_since(UNIX_EPOCH).is_err() {
            return Err(BeakIdError::InvalidEpoch(format!("{epoch:?}")));
        }

        Ok(Self { epoch, worker_id })
    }

    /// Reads `BEAKID_EPOCH` and `BEAKID_WORKER_ID` from the process
    /// environment.
    ///
    /// `BEAKID_EPOCH` is required and must be an RFC 3339 UTC datetime such as
    /// `2025-01-01T00:00:00Z`. `BEAKID_WORKER_ID` defaults to `0`.
    pub fn from_env() -> Result<Self> {
        let epoch_value = env::var(EPOCH_ENV).map_err(|_| BeakIdError::MissingEpoch)?;
        let epoch = parse_epoch(&epoch_value)?;
        let worker_id = match env::var(WORKER_ID_ENV) {
            Ok(value) => parse_worker_id(&value)?,
            Err(env::VarError::NotPresent) => 0,
            Err(_) => return Err(BeakIdError::InvalidWorkerId(String::new())),
        };

        Self::new(epoch, worker_id)
    }

    /// Configured custom epoch.
    #[must_use]
    pub const fn epoch(&self) -> std::time::SystemTime {
        self.epoch
    }

    /// Epoch expressed as Unix 100ms units.
    #[must_use]
    pub fn epoch_100ms_units(&self) -> u64 {
        (self
            .epoch
            .duration_since(UNIX_EPOCH)
            .expect("validated epoch must not be before Unix epoch")
            .as_millis()
            / 100)
            .try_into()
            .expect("validated epoch 100ms units must fit into u64")
    }

    /// Epoch expressed as Unix milliseconds.
    #[must_use]
    pub fn epoch_millis(&self) -> u64 {
        self.epoch
            .duration_since(UNIX_EPOCH)
            .expect("validated epoch must not be before Unix epoch")
            .as_millis()
            .try_into()
            .expect("validated epoch milliseconds must fit into u64")
    }

    /// Worker id encoded in the low 10 bits of generated IDs.
    #[must_use]
    pub const fn worker_id(&self) -> u16 {
        self.worker_id
    }
}

fn parse_epoch(value: &str) -> Result<DateTime> {
    let epoch = DateTime::parse(value).map_err(|_| BeakIdError::InvalidEpoch(value.to_owned()))?;
    if !value.ends_with('Z') || !value.contains('T') {
        return Err(BeakIdError::InvalidEpoch(value.to_owned()));
    }
    Ok(epoch)
}

fn parse_worker_id(value: &str) -> Result<u16> {
    let worker_id = value
        .parse::<u16>()
        .map_err(|_| BeakIdError::InvalidWorkerId(value.to_owned()))?;
    if worker_id > MAX_WORKER_ID {
        return Err(BeakIdError::WorkerIdOutOfRange(worker_id));
    }
    Ok(worker_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_utc_epoch() {
        let epoch = parse_epoch("2025-01-01T00:00:00Z").unwrap();
        assert_eq!(Timestamp::from(epoch).as_i64(), 1_735_689_600_000);
    }

    #[test]
    fn parses_utc_epoch_with_milliseconds() {
        let epoch = parse_epoch("2025-01-01T00:00:00.123Z").unwrap();
        assert_eq!(Timestamp::from(epoch).as_i64(), 1_735_689_600_123);
    }

    #[test]
    fn rejects_non_utc_epoch() {
        assert!(matches!(
            parse_epoch("2025-01-01T03:00:00+03:00"),
            Err(BeakIdError::InvalidEpoch(_))
        ));
    }

    #[test]
    fn rejects_non_rfc3339_shape() {
        assert!(matches!(
            parse_epoch("2025-01-01Z"),
            Err(BeakIdError::InvalidEpoch(_))
        ));
        assert!(matches!(
            parse_epoch("2025-01-01T00:00Z"),
            Err(BeakIdError::InvalidEpoch(_))
        ));
    }

    #[test]
    fn validates_worker_id_range() {
        assert_eq!(parse_worker_id("1023").unwrap(), 1023);
        assert_eq!(
            parse_worker_id("1024"),
            Err(BeakIdError::WorkerIdOutOfRange(1024))
        );
    }
}
