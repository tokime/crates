//! BeakId value type.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::generator::{MAX_TIMESTAMP, SEQUENCE_BITS, WORKER_BITS};

const WORKER_MASK: u64 = (1_u64 << WORKER_BITS) - 1;
const SEQUENCE_MASK: u64 = (1_u64 << SEQUENCE_BITS) - 1;
const SEQUENCE_SHIFT: u32 = WORKER_BITS;
const TIMESTAMP_SHIFT: u32 = WORKER_BITS + SEQUENCE_BITS;

/// A unique 64-bit identifier produced by [`crate::Generator`].
///
/// Values are always non-negative and can be stored in PostgreSQL `BIGINT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    /// Returns BeakId from 11 len base62 string
    pub fn from_base62(s: &str) -> Result<Self, crate::BeakIdError> {
        const LEN: usize = 11;

        if s.len() != LEN {
            return Err(crate::BeakIdError::InvalidBase62);
        }

        let mut n = 0i64;
        for &byte in s.as_bytes() {
            let digit = match byte {
                b'0'..=b'9' => (byte - b'0') as i64,
                b'A'..=b'Z' => (byte - b'A' + 10) as i64,
                b'a'..=b'z' => (byte - b'a' + 36) as i64,
                _ => return Err(crate::BeakIdError::InvalidBase62),
            };
            n = n
                .checked_mul(62)
                .and_then(|n| n.checked_add(digit))
                .ok_or(crate::BeakIdError::InvalidBase62)?;
        }

        Ok(BeakId(n))
    }

    /// Returns BeakId as base62 string with 11 len;
    pub fn base62(&self) -> String {
        const ALPHABET: &[u8; 62] =
            b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        const LEN: usize = 11;

        let mut buf = [b'0'; LEN];
        let mut n = self.0;

        for i in (0..LEN).rev() {
            buf[i] = ALPHABET[(n % 62) as usize];
            n /= 62;
        }

        // SAFETY: buf содержит только байты из ASCII-алфавита
        unsafe { String::from_utf8_unchecked(buf.to_vec()) }
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

impl Serialize for BeakId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            // Для JSON: кодируем в Base62 строку
            let encoded = self.base62();
            serializer.serialize_str(&encoded)
        } else {
            // Для Postcard: компактное число i64
            serializer.serialize_i64(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for BeakId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            // Для JSON: читаем строку и декодируем
            let s = String::deserialize(deserializer)?;
            Self::from_base62(&s).map_err(serde::de::Error::custom)
        } else {
            // Для Postcard: читаем как i64
            let val = i64::deserialize(deserializer)?;
            Ok(BeakId(val))
        }
    }
}
