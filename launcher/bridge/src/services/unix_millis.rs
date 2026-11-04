use std::time::{SystemTime, UNIX_EPOCH};

// The way times cross to the UI: whole milliseconds since 1970.
pub struct UnixMillis;

impl UnixMillis {
    // a time before 1970 reads as 0
    pub fn of(time: SystemTime) -> u64 {
        time.duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_millis() as u64).unwrap_or(0)
    }

    pub fn now() -> u64 {
        Self::of(SystemTime::now())
    }
}
