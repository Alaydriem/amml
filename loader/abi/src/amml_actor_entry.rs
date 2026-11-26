use crate::AmmlAabb;

/// One actor that `actors_in_v2` found: its box (a copy, read in the call), its `ActorType` (0 when the actor has no
/// type component) and whether it is the local player of the primary client.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AmmlActorEntry {
    pub aabb: AmmlAabb,
    pub actor_type: u32,
    pub is_local_player: u8,
    pub _reserved: [u8; 3],
}

const _: () = assert!(core::mem::size_of::<AmmlActorEntry>() == 32);
