use std::thread::ThreadId;

/// The call bookkeeping of one listener, guarded by the entry's mutex.
#[derive(Debug, Default)]
pub(crate) struct EntryState {
    pub(crate) removed: bool,
    pub(crate) calls: Vec<ThreadId>,
}
