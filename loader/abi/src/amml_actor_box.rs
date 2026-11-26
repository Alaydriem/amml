use crate::AmmlAabb;

/// One actor that `actors_in` found: its box (a copy, read in the call) and whether it is the local player of the
/// primary client.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AmmlActorBox {
    pub aabb: AmmlAabb,
    pub is_local_player: u8,
    pub _reserved: [u8; 3],
}

const _: () = assert!(core::mem::size_of::<AmmlActorBox>() == 28);
