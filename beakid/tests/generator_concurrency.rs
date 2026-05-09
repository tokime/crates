use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

use beakid::generator::decompose_id;
use beakid::{Config, Generator};
use tokistamp::DateTime;

#[test]
fn generator_produces_unique_ids_across_threads() {
    let config = Config::new(DateTime::from_unix_millis(0), 17).unwrap();
    let generator = Arc::new(Generator::from_config(config).unwrap());
    let ids = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for _ in 0..8 {
        let generator = Arc::clone(&generator);
        let ids = Arc::clone(&ids);
        handles.push(thread::spawn(move || {
            let mut local = Vec::with_capacity(2_000);
            for _ in 0..2_000 {
                local.push(generator.next_id().unwrap());
            }
            ids.lock().unwrap().extend(local);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let ids = ids.lock().unwrap();
    let unique = ids.iter().copied().collect::<HashSet<_>>();

    assert_eq!(ids.len(), 16_000);
    assert_eq!(unique.len(), ids.len());
    assert!(ids.iter().all(|id| id.as_i64() >> 63 == 0));
    assert!(ids.iter().all(|id| decompose_id(*id).2 == 17));
}
