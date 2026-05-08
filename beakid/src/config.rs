//! Environment and explicit configuration parsing.

use std::env;

use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, UtcOffset};

use crate::error::{BeakIdError, Result};
use crate::generator::MAX_WORKER_ID;

const EPOCH_ENV: &str = "BEAKID_EPOCH";
const WORKER_ID_ENV: &str = "BEAKID_WORKER_ID";

/// Runtime configuration for a [`crate::Generator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    epoch_100ms_units: u64,
    worker_id: u16,
}

impl Config {
    /// Builds configuration from explicit values.
    ///
    /// `epoch` must be a UTC datetime and `worker_id` must fit in 10 bits.
    pub fn new(epoch: OffsetDateTime, worker_id: u16) -> Result<Self> {
        if epoch.offset() != UtcOffset::UTC {
            return Err(BeakIdError::InvalidEpoch(
                epoch.format(&Rfc3339).unwrap_or_else(|_| epoch.to_string()),
            ));
        }
        if worker_id > MAX_WORKER_ID {
            return Err(BeakIdError::WorkerIdOutOfRange(worker_id));
        }

        let epoch_millis = epoch.unix_timestamp_nanos() / 1_000_000;
        if epoch_millis < 0 {
            return Err(BeakIdError::InvalidEpoch(
                epoch.format(&Rfc3339).unwrap_or_else(|_| epoch.to_string()),
            ));
        }

        Ok(Self {
            epoch_100ms_units: (epoch_millis as u64) / 100,
            worker_id,
        })
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

    /// Epoch expressed as Unix 100ms units.
    #[must_use]
    pub const fn epoch_100ms_units(&self) -> u64 {
        self.epoch_100ms_units
    }

    /// Worker id encoded in the low 10 bits of generated IDs.
    #[must_use]
    pub const fn worker_id(&self) -> u16 {
        self.worker_id
    }
}

fn parse_epoch(value: &str) -> Result<OffsetDateTime> {
    let epoch = OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| BeakIdError::InvalidEpoch(value.to_owned()))?;
    if epoch.offset() != UtcOffset::UTC {
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
        assert_eq!(epoch.offset(), UtcOffset::UTC);
    }

    #[test]
    fn rejects_non_utc_epoch() {
        assert!(matches!(
            parse_epoch("2025-01-01T03:00:00+03:00"),
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
