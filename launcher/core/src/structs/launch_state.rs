// How a launch ended its wait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchState {
    // the game showed a window
    Window { pid: u32, seconds: u64 },
    // no window in time; the game still runs
    NoWindow { pid: u32 },
    // the game ended before it showed a window
    Exited { code: Option<i32>, seconds: u64 },
}
