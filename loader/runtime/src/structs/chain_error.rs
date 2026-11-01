/// A refusal by the hook chain bookkeeping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ChainError {
    #[error("the hook limit of {0} is reached")]
    Full(usize),
    #[error("hook handle {0} is already in use")]
    DuplicateHandle(u64),
    #[error("original slot {0:#x} is already held by a live hook")]
    DuplicateSlot(u64),
    #[error("detour {0:#x} is already hooked on this target")]
    DuplicateDetour(u64),
    #[error("no hook has handle {0}")]
    UnknownHandle(u64),
}
