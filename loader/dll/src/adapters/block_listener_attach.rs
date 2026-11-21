/// A block source the loader's listener is attached to: the address of the listener object, the block source and its
/// dimension id, and the world epoch of the attach.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockListenerAttach {
    pub object: usize,
    pub source: usize,
    pub dimension: i32,
    pub epoch: u64,
}
