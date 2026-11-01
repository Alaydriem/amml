/// A log level this version of the contract defines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnownLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
