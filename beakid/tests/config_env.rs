use std::sync::Mutex;

use beakid::{BeakIdError, Config};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn config_reads_required_environment() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("BEAKID_EPOCH", "2025-01-01T00:00:00Z");
        std::env::set_var("BEAKID_WORKER_ID", "42");
    }

    let config = Config::from_env().unwrap();

    assert_eq!(config.worker_id(), 42);
    assert!(config.epoch_100ms_units() > 0);
}

#[test]
fn config_defaults_worker_id() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("BEAKID_EPOCH", "2025-01-01T00:00:00Z");
        std::env::remove_var("BEAKID_WORKER_ID");
    }

    let config = Config::from_env().unwrap();

    assert_eq!(config.worker_id(), 0);
}

#[test]
fn config_rejects_missing_epoch() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::remove_var("BEAKID_EPOCH");
        std::env::remove_var("BEAKID_WORKER_ID");
    }

    assert_eq!(Config::from_env(), Err(BeakIdError::MissingEpoch));
}
