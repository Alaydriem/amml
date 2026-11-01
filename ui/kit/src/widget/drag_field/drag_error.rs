/// Why a drag field refused the last value typed into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragError {
    NotANumber,
    OutOfRange,
}
