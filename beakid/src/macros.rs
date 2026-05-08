//! Runtime integration macros.

/// Defines a Tokio `main` function and starts BeakId's background updater.
///
/// This declarative macro is available with the `tokio-rt` feature. It is the
/// single-crate alternative to an attribute macro; Rust attribute procedural
/// macros must be provided by a separate `proc-macro` crate.
///
/// # Examples
///
/// ```ignore
/// beakid::tokio_main!({
///     let id = beakid::try_next_id()?;
///     println!("{id}");
///     Ok::<(), Box<dyn std::error::Error>>(())
/// });
/// ```
#[cfg(feature = "tokio-rt")]
#[macro_export]
macro_rules! tokio_main {
    ($body:block) => {
        #[tokio::main]
        async fn main() {
            $crate::start_tokio_background().expect("failed to start BeakId Tokio background task");
            $body
        }
    };
}

/// Spawns BeakId's background updater on the current Tokio runtime.
///
/// # Examples
///
/// ```ignore
/// #[tokio::main]
/// async fn main() -> Result<(), beakid::BeakIdError> {
///     beakid::tokio_spawn_background!()?;
///     Ok(())
/// }
/// ```
#[cfg(feature = "tokio-rt")]
#[macro_export]
macro_rules! tokio_spawn_background {
    () => {
        $crate::start_tokio_background()
    };
}
