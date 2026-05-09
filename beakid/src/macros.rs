//! Runtime integration macros.
//!
//! This crate intentionally has no async-runtime dependency. The macros below
//! expand to calls into the runtime selected by the application, so users must
//! depend on that runtime themselves.

/// Generates an ID from the singleton in a Tokio async context, yielding while
/// the singleton generator is blocked.
///
/// The macro retries on [`crate::BeakIdError::Blocked`] and calls
/// `tokio::task::yield_now().await` between attempts, allowing the background
/// updater task to advance real time.
///
/// # Examples
///
/// ```ignore
/// let id = beakid::tokio_next_id!();
/// ```
#[macro_export]
macro_rules! tokio_next_id {
    () => {{
        loop {
            match $crate::try_next_id() {
                Ok(id) => break id,
                Err($crate::BeakIdError::Blocked) => ::tokio::task::yield_now().await,
                Err(error) => return Err(error.into()),
            }
        }
    }};
}

/// Generates an ID from the singleton in a smol async context, yielding while
/// the singleton generator is blocked.
///
/// The macro retries on [`crate::BeakIdError::Blocked`] and calls
/// `smol::future::yield_now().await` between attempts, allowing the background
/// updater task to advance real time.
///
/// # Examples
///
/// ```ignore
/// let id = beakid::smol_next_id!();
/// ```
#[macro_export]
macro_rules! smol_next_id {
    () => {{
        loop {
            match $crate::try_next_id() {
                Ok(id) => break id,
                Err($crate::BeakIdError::Blocked) => ::smol::future::yield_now().await,
                Err(error) => return Err(error.into()),
            }
        }
    }};
}
