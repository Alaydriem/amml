/// A block the loader holds for a mod, as an owned resource id (slot, generation, kind). It is bound to the
/// mod that got it and to the world it was got in: after the mod's disable or a world leave it is stale.
/// `NONE` (0) is never handed out.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AmmlBlockHandle(pub u64);

impl AmmlBlockHandle {
    pub const NONE: Self = Self(0);
}
