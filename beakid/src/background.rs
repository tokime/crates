//! Background time-window hint updaters.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::error::{BeakIdError, Result};
use crate::generator::Generator;

const UPDATE_INTERVAL: Duration = Duration::from_millis(30);

/// Handle for a standard-thread background updater.
///
/// Dropping the handle asks the thread to stop and joins it.
#[derive(Debug)]
pub struct BackgroundHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl Drop for BackgroundHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

/// Starts a background updater thread for `generator`.
pub fn start_thread(generator: Arc<Generator>) -> Result<BackgroundHandle> {
    generator.refresh_hint()?;

    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let join = thread::Builder::new()
        .name("beakid-background".to_owned())
        .spawn(move || {
            while !thread_stop.load(Ordering::Acquire) {
                let _ = generator.refresh_hint();
                thread::sleep(UPDATE_INTERVAL);
            }
        })
        .map_err(|error| BeakIdError::BackgroundSpawnFailed(error.to_string()))?;

    Ok(BackgroundHandle {
        stop,
        join: Some(join),
    })
}
