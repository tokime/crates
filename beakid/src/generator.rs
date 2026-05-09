//! Core lock-free BeakId generation.

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
use std::time::Duration;

use crate::config::Config;
use crate::error::{BeakIdError, Result};
use tokistamp::Timestamp;

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
/// Maximum virtual lead over the real time window.
pub const MAX_VIRTUAL_WINDOWS: u64 = 10;

const WORKER_MASK: u64 = (1_u64 << WORKER_BITS) - 1;
const SEQUENCE_MASK: u64 = (1_u64 << SEQUENCE_BITS) - 1;
const SEQUENCE_SHIFT: u32 = WORKER_BITS;
const TIMESTAMP_SHIFT: u32 = WORKER_BITS + SEQUENCE_BITS;
const TIMESTAMP_MASK: i64 = !((1_i64 << TIMESTAMP_SHIFT) - 1);
const SEQUENCE_INCREMENT: i64 = 1_i64 << WORKER_BITS;

const STATE_BLOCKED: u64 = 1;
const STATE_UPDATING: u64 = 1 << 1;

/// A lock-free BeakId generator.
///
/// The generator keeps the next raw ID in one [`AtomicI64`]. Allocation
/// advances that atomic by `1 << WORKER_BITS`, incrementing the sequence while
/// preserving the worker-id bits. The epoch is stored as [`SystemTime`] and a
/// small state atomic tracks update and blocked states.
#[derive(Debug)]
pub struct Generator {
    id: AtomicI64,
    state: AtomicU64,
    epoch: SystemTime,
}

impl Generator {
    /// Creates a generator from validated configuration.
    pub fn from_config(config: Config) -> Result<Self> {
        Self::new(config.epoch(), config.worker_id())
    }

    /// Creates a generator from explicit parts.
    pub fn new(epoch: SystemTime, worker_id: u16) -> Result<Self> {
        if worker_id > MAX_WORKER_ID {
            return Err(BeakIdError::WorkerIdOutOfRange(worker_id));
        }

        let window = current_window(epoch)?;
        validate_window(window)?;

        Ok(Self {
            id: AtomicI64::new(raw_id(window, 0, worker_id)),
            state: AtomicU64::new(0),
            epoch,
        })
    }

    /// Returns the next unique ID as a non-negative `i64`.
    pub fn next_id(&self) -> Result<i64> {
        let state = loop {
            let state = self.state.load(Ordering::Acquire);
            if state & STATE_UPDATING == 0 {
                break state;
            }
            std::hint::spin_loop();
        };

        if state & STATE_BLOCKED != 0 {
            return Err(BeakIdError::Blocked);
        }

        let id = self.id.fetch_add(SEQUENCE_INCREMENT, Ordering::Relaxed);
        let next_id = id.wrapping_add(SEQUENCE_INCREMENT);

        if (id & TIMESTAMP_MASK) != (next_id & TIMESTAMP_MASK) {
            self.refresh_hint()?;
        }

        Ok(id)
    }

    /// Reconciles the generator's virtual time with real time.
    ///
    /// The background worker calls this every ~30ms. The hot path also calls it
    /// when sequence overflow crosses into a new virtual window.
    pub fn refresh_hint(&self) -> Result<u64> {
        let real_window = current_window(self.epoch)?;
        validate_window(real_window)?;

        let mut state = self.state.load(Ordering::Acquire);
        loop {
            if state & STATE_UPDATING != 0 {
                return Ok(real_window);
            }
            if state & STATE_BLOCKED != 0 {
                let timestamp = timestamp_part(real_window);
                let id = self.id.load(Ordering::Relaxed);
                if id > timestamp {
                    let delta = ((id - timestamp) as u64) >> TIMESTAMP_SHIFT;
                    if delta >= 8 {
                        return Ok(real_window);
                    }
                }
                self.state.store(0, Ordering::Release);
                return Ok(real_window);
            }
            match self.state.compare_exchange_weak(
                state,
                STATE_UPDATING | STATE_BLOCKED,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(current) => state = current,
            }
        }

        self.reconcile_id_with_time(real_window);
        Ok(real_window)
    }

    /// Returns the current real time window from the configured epoch.
    pub fn real_window_hint(&self) -> Result<u64> {
        current_window(self.epoch)
    }

    /// Returns the configured worker id encoded in the low 10 bits.
    #[must_use]
    pub fn worker_id(&self) -> u16 {
        (self.id.load(Ordering::Acquire) as u64 & WORKER_MASK) as u16
    }

    /// Returns the configured epoch.
    #[must_use]
    pub const fn epoch(&self) -> SystemTime {
        self.epoch
    }

    /// Returns the absolute creation timestamp encoded in `id`.
    ///
    /// BeakId stores only a 100ms window offset from the configured epoch. This
    /// method uses the generator's epoch to reconstruct a [`tokistamp::Timestamp`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::UNIX_EPOCH;
    ///
    /// let generator = beakid::Generator::new(UNIX_EPOCH, 1)?;
    /// let id = generator.next_id()?;
    /// let created_at = generator.timestamp(id)?;
    ///
    /// assert!(created_at.as_i64() >= 0);
    /// # Ok::<(), beakid::BeakIdError>(())
    /// ```
    pub fn timestamp(&self, id: i64) -> Result<Timestamp> {
        let window = timestamp_window(id);
        let offset_millis = window
            .checked_mul(100)
            .ok_or(BeakIdError::TimestampOverflow(window))?;
        let epoch_millis = self
            .epoch
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BeakIdError::ClockBeforeEpoch)?
            .as_millis();
        let millis = epoch_millis
            .checked_add(offset_millis as u128)
            .ok_or(BeakIdError::TimestampOverflow(window))?;
        let millis = i64::try_from(millis).map_err(|_| BeakIdError::TimestampOverflow(window))?;

        Ok(Timestamp::from_millis(millis))
    }

    fn reconcile_id_with_time(&self, real_window: u64) {
        let timestamp = timestamp_part(real_window);
        let mut id = self.id.load(Ordering::Relaxed);
        loop {
            if id > timestamp {
                let delta = ((id - timestamp) as u64) >> TIMESTAMP_SHIFT;
                if delta >= MAX_VIRTUAL_WINDOWS {
                    self.state.store(STATE_BLOCKED, Ordering::Release);
                    return;
                }
                break;
            }

            let reset_id = (id & WORKER_MASK as i64) | timestamp;
            match self
                .id
                .compare_exchange_weak(id, reset_id, Ordering::Release, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(current) => id = current,
            }
        }

        self.state.store(0, Ordering::Release);
    }
}

/// Constructs the final 64-bit ID from already validated parts.
#[must_use]
pub const fn construct_id(timestamp: u64, sequence: u64, worker_id: u16) -> i64 {
    raw_id(timestamp, sequence, worker_id)
}

/// Decodes an ID into `(timestamp, sequence, worker_id)`.
#[must_use]
pub const fn decompose_id(id: i64) -> (u64, u64, u16) {
    let raw = id as u64;
    (
        timestamp_window(id),
        (raw >> SEQUENCE_SHIFT) & SEQUENCE_MASK,
        (raw & WORKER_MASK) as u16,
    )
}

const fn raw_id(timestamp: u64, sequence: u64, worker_id: u16) -> i64 {
    ((timestamp << TIMESTAMP_SHIFT)
        | ((sequence & SEQUENCE_MASK) << SEQUENCE_SHIFT)
        | ((worker_id as u64) & WORKER_MASK)) as i64
}

const fn timestamp_part(window: u64) -> i64 {
    (window << TIMESTAMP_SHIFT) as i64
}

const fn timestamp_window(id: i64) -> u64 {
    ((id as u64) >> TIMESTAMP_SHIFT) & MAX_TIMESTAMP
}

fn current_window(epoch: SystemTime) -> Result<u64> {
    let elapsed = SystemTime::now()
        .duration_since(epoch)
        .map_err(|_| BeakIdError::ClockBeforeEpoch)?;
    let window = elapsed.as_millis() / 100;
    u64::try_from(window).map_err(|_| BeakIdError::TimestampOverflow(u64::MAX))
}

fn validate_window(window: u64) -> Result<()> {
    if window > MAX_TIMESTAMP {
        Err(BeakIdError::TimestampOverflow(window))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_generator(window: u64, sequence: u64, worker_id: u16) -> Generator {
        test_generator_with_real_window(window, sequence, worker_id, window)
    }

    fn test_generator_with_real_window(
        window: u64,
        sequence: u64,
        worker_id: u16,
        real_window: u64,
    ) -> Generator {
        Generator {
            id: AtomicI64::new(raw_id(window, sequence, worker_id)),
            state: AtomicU64::new(0),
            epoch: SystemTime::now() - Duration::from_millis(real_window * 100),
        }
    }

    #[test]
    fn constructs_expected_layout() {
        let id = construct_id(0b101, 0b11, 42);
        assert_eq!(id >> 63, 0);
        assert_eq!(decompose_id(id), (0b101, 0b11, 42));
    }

    #[test]
    fn first_id_uses_sequence_zero() {
        let generator = test_generator(100, 0, 7);

        let id = generator.next_id().unwrap();
        assert_eq!(decompose_id(id), (100, 0, 7));
    }

    #[test]
    fn sequence_overflow_advances_virtual_window() {
        let generator = test_generator(100, MAX_SEQUENCE, 9);

        let last_in_window = generator.next_id().unwrap();
        let first_virtual = generator.next_id().unwrap();

        assert_eq!(decompose_id(last_in_window), (100, MAX_SEQUENCE, 9));
        assert_eq!(decompose_id(first_virtual), (101, 0, 9));
    }

    #[test]
    fn refresh_resets_to_real_window_when_time_advances() {
        let mut generator = test_generator(100, 55, 3);
        generator.epoch = SystemTime::now() - Duration::from_millis(101 * 100);

        generator.refresh_hint().unwrap();
        let id = generator.next_id().unwrap();

        assert_eq!(decompose_id(id), (101, 0, 3));
    }

    #[test]
    fn timestamp_uses_generator_epoch() {
        let generator = Generator::new(UNIX_EPOCH, 1).unwrap();
        let id = construct_id(123, 45, 1);

        let timestamp = generator.timestamp(id).unwrap();

        assert_eq!(timestamp.as_i64(), 12_300);
    }

    #[test]
    fn refresh_blocks_when_virtual_time_is_exhausted() {
        let generator = test_generator_with_real_window(110, 0, 3, 100);

        generator.refresh_hint().unwrap();

        assert_eq!(generator.next_id(), Err(BeakIdError::Blocked));
    }

    #[test]
    fn blocked_generator_unblocks_after_real_time_catches_up() {
        let mut generator = test_generator_with_real_window(110, 0, 3, 100);
        generator.refresh_hint().unwrap();
        assert_eq!(generator.next_id(), Err(BeakIdError::Blocked));

        generator.epoch = SystemTime::now() - Duration::from_millis(102 * 100);
        generator.refresh_hint().unwrap();

        assert_eq!(generator.next_id(), Err(BeakIdError::Blocked));

        generator.epoch = SystemTime::now() - Duration::from_millis(103 * 100);
        generator.refresh_hint().unwrap();

        assert!(generator.next_id().is_ok());
    }
}
