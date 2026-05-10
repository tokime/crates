//! BeakId value type.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::generator::{MAX_TIMESTAMP, SEQUENCE_BITS, WORKER_BITS};

const WORKER_MASK: u64 = (1_u64 << WORKER_BITS) - 1;
const SEQUENCE_MASK: u64 = (1_u64 << SEQUENCE_BITS) - 1;
const SEQUENCE_SHIFT: u32 = WORKER_BITS;
const TIMESTAMP_SHIFT: u32 = WORKER_BITS + SEQUENCE_BITS;

/// A unique 64-bit identifier produced by [`crate::Generator`].
///
/// Values are always non-negative and can be stored in PostgreSQL `BIGINT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BeakId(i64);

impl BeakId {
    /// Wraps a raw `i64` value as `BeakId`.
    ///
    /// This is useful when reconstructing IDs read from storage. It does not
    /// validate that the value was produced by this crate.
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// Returns the raw `i64` value for PostgreSQL `BIGINT` storage.
    #[must_use]
    pub const fn as_i64(self) -> i64 {
        self.0
    }

    /// Returns the 100ms timestamp window encoded in the ID.
    #[must_use]
    pub const fn timestamp_window(self) -> u64 {
        ((self.0 as u64) >> TIMESTAMP_SHIFT) & MAX_TIMESTAMP
    }

    /// Returns the sequence value encoded in the ID.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        ((self.0 as u64) >> SEQUENCE_SHIFT) & SEQUENCE_MASK
    }

    /// Returns the worker id encoded in the ID.
    #[must_use]
    pub const fn worker_id(self) -> u16 {
        (self.0 as u64 & WORKER_MASK) as u16
    }

    /// Decodes the ID into `(timestamp_window, sequence, worker_id)`.
    #[must_use]
    pub const fn parts(self) -> (u64, u64, u16) {
        (self.timestamp_window(), self.sequence(), self.worker_id())
    }
}

impl From<i64> for BeakId {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

impl From<BeakId> for i64 {
    fn from(id: BeakId) -> Self {
        id.as_i64()
    }
}

impl fmt::Display for BeakId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
