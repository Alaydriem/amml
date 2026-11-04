use std::sync::{Mutex, MutexGuard};

// A lock the app takes even when a thread that held it panicked: the data behind it is plain state that is
// replaced whole, so a poisoned lock is no reason to stop the app.
pub struct Locked;

impl Locked {
    pub fn of<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        match mutex.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}
