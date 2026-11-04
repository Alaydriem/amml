// The state of one window enumeration: the process looked for, and whether a visible window of it was seen.
#[derive(Debug, Clone, Copy)]
pub struct WindowSearch {
    pub pid: u32,
    pub found: bool,
}
