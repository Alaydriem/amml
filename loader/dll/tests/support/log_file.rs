use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

/// A log file that is written by a background thread, so reads wait for text to appear.
pub struct LogFile;

impl LogFile {
    pub fn wait_for(path: &Path, needle: &str) -> String {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let text = fs::read_to_string(path).unwrap_or_default();
            if text.contains(needle) || Instant::now() > deadline {
                return text;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
