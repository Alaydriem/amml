use std::time::SystemTime;

// A game the launcher started and still holds the process handle of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningGame {
    pub pid: u32,
    pub instance: String,
    pub started: SystemTime,
}
