use crate::AmmlFace;

/// The pose of a skull ghost: the face of the block it stands on or hangs from (`UP` for a skull on the floor, a side
/// face for a skull on a wall) and its turn in sixteenths of a full turn (0 to 15, used on the floor only).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlSkullPose {
    pub face: AmmlFace,
    pub turn: u8,
}

const _: () = assert!(core::mem::size_of::<AmmlSkullPose>() == 2);
