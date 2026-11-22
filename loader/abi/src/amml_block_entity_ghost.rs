/// A block entity ghost: a game block actor the loader owns, bound to one mod and one world. An id of an older
/// generation, or one that was freed, is `STALE`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlBlockEntityGhost {
    pub id: u32,
    pub generation: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlBlockEntityGhost>() == 8);
