use crate::{AmmlBlockPos, AmmlFace, AmmlVec3};

/// What the crosshair points at, from the game's own hit result of the level (`ActorHitResultFn`): `kind` `BLOCK` with
/// the face the pick entered, the block and the point on it; `ENTITY` with the point only; `NOTHING` (also an entity
/// out of reach) with every other field zero.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AmmlHitResult {
    pub kind: u8,
    pub face: AmmlFace,
    pub _reserved: [u8; 2],
    pub pos: AmmlBlockPos,
    pub point: AmmlVec3,
}

impl AmmlHitResult {
    pub const NOTHING: u8 = 0;
    pub const BLOCK: u8 = 1;
    pub const ENTITY: u8 = 2;
}

const _: () = assert!(core::mem::size_of::<AmmlHitResult>() == 28);
