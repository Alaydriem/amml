use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::AmmlActorTable;

/// The dimension of a `LocalPlayer` (`LocalPlayer::dimension`), as an opaque handle. It lives no longer than the
/// event that gave its player; the loader checks on every call that it is still the player's dimension.
pub struct Dimension<'f> {
    handle: *const c_void,
    actor: AmmlActorTable,
    _event: PhantomData<&'f ()>,
}

impl Dimension<'_> {
    pub const OVERWORLD: i32 = 0;
    pub const NETHER: i32 = 1;
    pub const THE_END: i32 = 2;

    pub(crate) fn new(handle: *const c_void, actor: AmmlActorTable) -> Self {
        Self { handle, actor, _event: PhantomData }
    }

    /// The dimension id: `OVERWORLD`, `NETHER`, `THE_END`, or another value for a custom dimension. `None` when
    /// the loader refuses the read (the player changed dimension since, for example). Area `actor.read` version 2.
    pub fn id(&self) -> Option<i32> {
        let read = self.actor.dimension_id?;
        let mut out = 0i32;
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then_some(out)
    }
}

impl PartialEq for Dimension<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
