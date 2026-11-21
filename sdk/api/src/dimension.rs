use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{AmmlActorTable, AmmlWorldTable};

/// The dimension of a `LocalPlayer` (`LocalPlayer::dimension`), as an opaque handle. It lives no longer than the
/// event that gave its player; the loader checks on every call that it is still the player's dimension.
pub struct Dimension<'f> {
    handle: *const c_void,
    actor: AmmlActorTable,
    world: Option<AmmlWorldTable>,
    _event: PhantomData<&'f ()>,
}

impl Dimension<'_> {
    pub const OVERWORLD: i32 = 0;
    pub const NETHER: i32 = 1;
    pub const THE_END: i32 = 2;

    pub(crate) fn new(handle: *const c_void, actor: AmmlActorTable, world: Option<AmmlWorldTable>) -> Self {
        Self { handle, actor, world, _event: PhantomData }
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

    /// The sky darkness of the dimension for the current time of day: 0 in full daylight, higher at night (up to 15).
    /// `None` when the loader refuses the read (the player changed dimension since, for example) or does not serve
    /// the area. Area `world.sky_darken` (`Area::WORLD_SKY_DARKEN`).
    pub fn sky_darken(&self) -> Option<u8> {
        let read = self.world?.sky_darken;
        let mut out = 0u8;
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
