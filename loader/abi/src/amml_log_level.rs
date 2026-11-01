use crate::KnownLogLevel;

/// Severity of a log line sent through `AmmlApi::log`.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlLogLevel(pub u8);

impl AmmlLogLevel {
    pub const TRACE: Self = Self(0);
    pub const DEBUG: Self = Self(1);
    pub const INFO: Self = Self(2);
    pub const WARN: Self = Self(3);
    pub const ERROR: Self = Self(4);

    /// The level as a Rust enum, or `None` for a value this version does not define.
    pub fn known(self) -> Option<KnownLogLevel> {
        match self {
            Self::TRACE => Some(KnownLogLevel::Trace),
            Self::DEBUG => Some(KnownLogLevel::Debug),
            Self::INFO => Some(KnownLogLevel::Info),
            Self::WARN => Some(KnownLogLevel::Warn),
            Self::ERROR => Some(KnownLogLevel::Error),
            _ => None,
        }
    }
}
