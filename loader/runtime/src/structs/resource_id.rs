/// The handle of one owned resource: bits 0..32 slot, 32..56 generation, 56..64 kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u64);
