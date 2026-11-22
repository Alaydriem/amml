use crate::BlockHandle;

/// The block `Blocks::from_state` found. `exact` is true when the block holds every state asked for with the value
/// asked for; false when the game left out a state the block type does not have or a value that does not fit it (the
/// block then has the default value of that state).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockMatch {
    pub block: BlockHandle,
    pub exact: bool,
}
