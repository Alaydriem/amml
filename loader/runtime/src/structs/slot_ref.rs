/// One slot of an item stack request: the container id of the request protocol, the slot, and the net id of the
/// stack the slot holds (0 for an empty slot).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotRef {
    pub container: u8,
    pub slot: u8,
    pub net_id: i32,
}
