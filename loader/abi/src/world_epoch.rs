/// Counts the worlds the player joined. Epoch 0 means "not bound to a world".
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldEpoch(pub u64);
