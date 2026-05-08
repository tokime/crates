#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Public API for BeakId.

pub mod background;
pub mod config;
pub mod error;
pub mod generator;
pub mod macros;

use std::sync::{Arc, OnceLock};

pub use background::BackgroundHandle;
pub use config::Config;
pub use error::{BeakIdError, Result};
pub use generator::Generator;

static GENERATOR: OnceLock<Result<Arc<Generator>>> = OnceLock::new();
static BACKGROUND: OnceLock<BackgroundHandle> = OnceLock::new();

fn singleton() -> Result<Arc<Generator>> {
    GENERATOR
        .get_or_init(|| {
            Config::from_env()
                .and_then(Generator::from_config)
                .map(Arc::new)
        })
        .as_ref()
        .map(Arc::clone)
        .map_err(Clone::clone)
}

/// Returns the next process-wide BeakId.
///
/// The global generator is initialized lazily on first use from environment
/// variables. Use [`try_next_id`] when configuration errors should be handled
/// instead of panicking.
///
/// # Panics
///
/// Panics if `BEAKID_EPOCH` is missing or invalid, if `BEAKID_WORKER_ID` is
/// invalid, or if the system clock is before the configured epoch.
///
/// # Examples
///
/// ```no_run
/// # unsafe {
/// std::env::set_var("BEAKID_EPOCH", "2025-01-01T00:00:00Z");
/// std::env::set_var("BEAKID_WORKER_ID", "42");
/// # }
/// let id = beakid::next_id();
/// assert_eq!(id >> 63, 0);
/// ```
#[must_use]
pub fn next_id() -> u64 {
    try_next_id().expect("failed to generate BeakId")
}

/// Returns the next process-wide BeakId, reporting configuration and clock
/// errors explicitly.
///
/// # Examples
///
/// ```no_run
/// # unsafe {
/// std::env::set_var("BEAKID_EPOCH", "2025-01-01T00:00:00Z");
/// # }
/// let id = beakid::try_next_id()?;
/// # Ok::<(), beakid::BeakIdError>(())
/// ```
pub fn try_next_id() -> Result<u64> {
    singleton()?.next_id()
}

/// Starts the singleton background updater on a standard OS thread.
///
/// The updater refreshes the real 100ms time-window hint roughly every 30ms.
/// Calling this function more than once is harmless. If this function is not
/// called, ID generation still works at normal rates, but extremely high
/// sequence overflow workloads can block waiting for the hint to advance.
///
/// # Examples
///
/// ```no_run
/// # unsafe {
/// std::env::set_var("BEAKID_EPOCH", "2025-01-01T00:00:00Z");
/// # }
/// beakid::start_background()?;
/// let id = beakid::try_next_id()?;
/// # Ok::<(), beakid::BeakIdError>(())
/// ```
pub fn start_background() -> Result<()> {
    let generator = singleton()?;
    let handle = background::start_thread(generator)?;
    let _ = BACKGROUND.set(handle);
    Ok(())
}

/// Starts the singleton background updater on the current Tokio runtime.
///
/// The task is detached and runs until the runtime shuts down.
#[cfg(feature = "tokio-rt")]
pub fn start_tokio_background() -> Result<()> {
    let generator = singleton()?;
    background::start_tokio_task(generator);
    Ok(())
}
