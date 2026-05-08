//! Core lock-free BeakId generation.

use std::hint;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::error::{BeakIdError, Result};

/// Number of bits used by the timestamp component.
pub const TIMESTAMP_BITS: u32 = 35;
/// Number of bits used by the per-window sequence component.
pub const SEQUENCE_BITS: u32 = 18;
/// Number of bits used by the worker component.
pub const WORKER_BITS: u32 = 10;

/// Largest encodable timestamp window.
pub const MAX_TIMESTAMP: u64 = (1_u64 << TIMESTAMP_BITS) - 1;
/// Largest encodable sequence value.
pub const MAX_SEQUENCE: u64 = (1_u64 << SEQUENCE_BITS) - 1;
/// Largest encodable worker id.
pub const MAX_WORKER_ID: u16 = (1_u16 << WORKER_BITS) - 1;
/// Maximum virtual lead over the real time-window hint.
pub const MAX_VIRTUAL_WINDOWS: u64 = 10;

const WORKER_MASK: u64 = (1_u64 << WORKER_BITS) - 1;
const SEQUENCE_MASK: u64 = (1_u64 << SEQUENCE_BITS) - 1;
const WORKER_SHIFT: u32 = 0;
const SEQUENCE_SHIFT: u32 = WORKER_BITS;
const TIMESTAMP_SHIFT: u32 = WORKER_BITS + SEQUENCE_BITS;

/// A lock-free BeakId generator.
///
/// The generator is thread-safe and uses atomics only. IDs are laid out as:
///
/// ```text
/// [ reserved: 1 | timestamp: 35 | sequence: 18 | worker: 10 ]
/// ```
#[derive(Debug)]
pub struct Generator {
    epoch_100ms_units: u64,
    worker_id: u16,
    state: AtomicU64,
    real_window_hint: AtomicU64,
}

impl Generator {
    /// Creates a generator from validated configuration.
    pub fn from_config(config: Config) -> Result<Self> {
        let real_window = current_window(config.epoch_100ms_units())?;
        Ok(Self {
            epoch_100ms_units: config.epoch_100ms_units(),
            worker_id: config.worker_id(),
            state: AtomicU64::new(pack_state(real_window, 0)),
            real_window_hint: AtomicU64::new(real_window),
        })
    }

    /// Creates a generator from explicit parts.
    pub fn new(epoch_100ms_units: u64, worker_id: u16) -> Result<Self> {
        Generator::from_config(Config::new(
            time::OffsetDateTime::from_unix_timestamp_nanos(
                (epoch_100ms_units as i128) * 100_000_000,
            )
            .map_err(|_| BeakIdError::InvalidEpoch(epoch_100ms_units.to_string()))?,
            worker_id,
        )?)
    }

    /// Returns the next unique ID.
    ///
    /// This method performs no locks and uses the atomically refreshed real
    /// window hint. If the sequence overflows and virtual time is already 10
    /// windows ahead of the hint, it waits until the hint catches up.
    pub fn next_id(&self) -> Result<u64> {
        loop {
            let state = self.state.load(Ordering::Acquire);
            let (window, sequence) = unpack_state(state);
            let real_window = self.real_window_hint.load(Ordering::Acquire);

            let (id_window, id_sequence, next_window, next_sequence) = if real_window > window {
                (real_window, 0, real_window, 1)
            } else if sequence < MAX_SEQUENCE {
                (window, sequence, window, sequence + 1)
            } else {
                let candidate = window + 1;
                if candidate <= real_window.saturating_add(MAX_VIRTUAL_WINDOWS) {
                    (window, sequence, candidate, 0)
                } else {
                    self.wait_for_hint_after(real_window)?;
                    continue;
                }
            };

            validate_window(id_window)?;
            validate_window(next_window)?;
            let next_state = pack_state(next_window, next_sequence);
            if self
                .state
                .compare_exchange_weak(state, next_state, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(construct_id(id_window, id_sequence, self.worker_id));
            }

            hint::spin_loop();
        }
    }

    /// Refreshes the real-time hint from the system clock.
    pub fn refresh_hint(&self) -> Result<u64> {
        let window = current_window(self.epoch_100ms_units)?;
        validate_window(window)?;
        self.real_window_hint.store(window, Ordering::Release);
        Ok(window)
    }

    /// Returns the latest real-time hint known to this generator.
    #[must_use]
    pub fn real_window_hint(&self) -> u64 {
        self.real_window_hint.load(Ordering::Acquire)
    }

    /// Returns the configured worker id.
    #[must_use]
    pub const fn worker_id(&self) -> u16 {
        self.worker_id
    }

    /// Returns the configured epoch as Unix 100ms units.
    #[must_use]
    pub const fn epoch_100ms_units(&self) -> u64 {
        self.epoch_100ms_units
    }

    fn wait_for_hint_after(&self, old_hint: u64) -> Result<()> {
        let mut spins = 0_u32;
        loop {
            let hint_now = self.real_window_hint.load(Ordering::Acquire);
            if hint_now > old_hint {
                return Ok(());
            }
            if spins < 64 {
                spins += 1;
                hint::spin_loop();
            } else {
                thread::yield_now();
                thread::sleep(Duration::from_millis(1));
            }
        }
    }
}

/// Constructs the final 64-bit ID from already validated parts.
#[must_use]
pub const fn construct_id(timestamp: u64, sequence: u64, worker_id: u16) -> u64 {
    (timestamp << TIMESTAMP_SHIFT)
        | ((sequence & SEQUENCE_MASK) << SEQUENCE_SHIFT)
        | (((worker_id as u64) & WORKER_MASK) << WORKER_SHIFT)
}

/// Decodes an ID into `(timestamp, sequence, worker_id)`.
#[must_use]
pub const fn decompose_id(id: u64) -> (u64, u64, u16) {
    (
        (id >> TIMESTAMP_SHIFT) & MAX_TIMESTAMP,
        (id >> SEQUENCE_SHIFT) & SEQUENCE_MASK,
        (id & WORKER_MASK) as u16,
    )
}

fn current_window(epoch_100ms_units: u64) -> Result<u64> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| BeakIdError::ClockBeforeEpoch)?;
    let current_100ms_units = elapsed.as_millis() / 100;
    let current_100ms_units =
        u64::try_from(current_100ms_units).map_err(|_| BeakIdError::TimestampOverflow(u64::MAX))?;
    current_100ms_units
        .checked_sub(epoch_100ms_units)
        .ok_or(BeakIdError::ClockBeforeEpoch)
}

fn validate_window(window: u64) -> Result<()> {
    if window > MAX_TIMESTAMP {
        Err(BeakIdError::TimestampOverflow(window))
    } else {
        Ok(())
    }
}

const fn pack_state(window: u64, sequence: u64) -> u64 {
    (window << SEQUENCE_BITS) | (sequence & SEQUENCE_MASK)
}

const fn unpack_state(state: u64) -> (u64, u64) {
    (state >> SEQUENCE_BITS, state & SEQUENCE_MASK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_expected_layout() {
        let id = construct_id(0b101, 0b11, 42);
        assert_eq!(id >> 63, 0);
        assert_eq!(decompose_id(id), (0b101, 0b11, 42));
    }

    #[test]
    fn state_round_trips() {
        let packed = pack_state(123, 456);
        assert_eq!(unpack_state(packed), (123, 456));
    }

    #[test]
    fn first_id_uses_sequence_zero() {
        let generator = Generator {
            epoch_100ms_units: 0,
            worker_id: 7,
            state: AtomicU64::new(pack_state(100, 0)),
            real_window_hint: AtomicU64::new(100),
        };

        let id = generator.next_id().unwrap();
        assert_eq!(decompose_id(id), (100, 0, 7));
    }

    #[test]
    fn sequence_overflow_advances_virtual_window() {
        let generator = Generator {
            epoch_100ms_units: 0,
            worker_id: 9,
            state: AtomicU64::new(pack_state(100, MAX_SEQUENCE)),
            real_window_hint: AtomicU64::new(100),
        };

        let last_in_window = generator.next_id().unwrap();
        let first_virtual = generator.next_id().unwrap();

        assert_eq!(decompose_id(last_in_window), (100, MAX_SEQUENCE, 9));
        assert_eq!(decompose_id(first_virtual), (101, 0, 9));
    }
}
