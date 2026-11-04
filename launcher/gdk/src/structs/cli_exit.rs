#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliExit {
    Ok,
    Error,
    NotReady,
    RestoreFailed,
}

impl CliExit {
    pub fn code(&self) -> i32 {
        match self {
            Self::Ok => 0,
            Self::Error => 1,
            Self::NotReady => 2,
            Self::RestoreFailed => 3,
        }
    }
}
