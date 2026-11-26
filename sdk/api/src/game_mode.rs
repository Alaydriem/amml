use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{AmmlActorTable, AmmlBlockPos, AmmlFace, AmmlStr, AmmlVec3};

use crate::{ApiError, BlockPos, Face, Vec3};

/// The game mode of a `LocalPlayer`: its own build, its build repeat delay and its pick range. It lives no longer
/// than the event that gave its client.
///
/// The calls go through the loader's actor table, on the game thread; the loader checks on every call that the player
/// is still the local player of the primary client.
pub struct GameMode<'f> {
    player: *const c_void,
    actor: AmmlActorTable,
    module: &'static str,
    _event: PhantomData<&'f ()>,
}

impl<'f> GameMode<'f> {
    pub(crate) fn new(player: *const c_void, actor: AmmlActorTable, module: &'static str) -> Self {
        Self { player, actor, module, _event: PhantomData }
    }

    /// Builds with the main hand at the face `face` of the block `pos`, as a click at `click` (a point in world
    /// coordinates on that face) would: the new block goes next to `pos`. `Ok(true)` when the game built, `Ok(false)`
    /// when it did not (a listener of `BuildBlock` may have cancelled it). The `BuildBlock` this raises comes on the
    /// same stack, before this returns, with the origin `THIS_MOD` for this mod's listeners: release every lock the
    /// listener takes before the call. `Err(Loader(INVALID_ARGUMENT))` for a face that is not one of the six or a
    /// click that is not finite; `Err(Refused)` off the game thread; `Err(NotAvailable)` without area `player.build`
    /// (`Area::PLAYER_BUILD`).
    pub fn build_at(&self, pos: BlockPos, face: Face, click: Vec3) -> Result<bool, ApiError> {
        let build = self.actor.build_at.ok_or(ApiError::NotAvailable)?;
        let pos = AmmlBlockPos::from(pos);
        let click = AmmlVec3::from(click);
        let mut out = 0u8;
        // SAFETY: the loader checks the handle before it reads game memory; the module id, the position, the click and
        // `out` live for the call.
        #[allow(unsafe_code)]
        let status =
            unsafe { build(AmmlStr::from_str(self.module), self.player, &pos, AmmlFace::from(face), &click, &mut out) };
        ApiError::check(status).map(|()| out != 0)
    }

    /// The game mode's build repeat delay, in milliseconds. Area `player.build` (`Area::PLAYER_BUILD`).
    pub fn build_delay(&self) -> Option<u32> {
        let read = self.actor.build_delay?;
        let mut out = 0u32;
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.player, &mut out) };
        status.is_ok().then_some(out)
    }

    /// The largest distance at which the player can pick a block, in blocks (more in creative). Area
    /// `player.pick_range` (`Area::PLAYER_PICK_RANGE`).
    pub fn max_pick_range(&self) -> Option<f32> {
        let read = self.actor.max_pick_range?;
        let mut out = 0.0f32;
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.player, &mut out) };
        status.is_ok().then_some(out)
    }
}
