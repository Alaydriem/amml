/// What the loader read of one player slot for a request: the net id of the stack, its count, its stack limit and
/// its identity key (0 when the identity is not known).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestSlot {
    pub net_id: i32,
    pub count: u8,
    pub max: u8,
    pub key: u64,
}
