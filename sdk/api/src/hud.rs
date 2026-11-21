use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::AmmlClientTable;

use crate::ApiError;

/// The game HUD of a `Client` (`Client::hud`), under the client call rules. It lives no longer than its client.
pub struct Hud<'f> {
    table: AmmlClientTable,
    handle: *const c_void,
    _event: PhantomData<&'f ()>,
}

impl Hud<'_> {
    /// The hotbar slots `select_hotbar_slot` takes: 0 to `HOTBAR_SLOTS - 1`.
    pub const HOTBAR_SLOTS: u8 = 9;

    pub(crate) fn new(table: AmmlClientTable, handle: *const c_void) -> Self {
        Self { table, handle, _event: PhantomData }
    }

    /// Selects hotbar slot `slot` (0 is the leftmost) as the game's own HUD does, with no key. `Refused` for a slot
    /// of `HOTBAR_SLOTS` or above, and while the game shows no HUD (the HUD was not ticked in the last half second);
    /// `NotAvailable` without area `hud.select_hotbar`.
    pub fn select_hotbar_slot(&self, slot: u8) -> Result<(), ApiError> {
        let select = self.table.select_hotbar_slot.ok_or(ApiError::NotAvailable)?;
        // SAFETY: the loader checks the handle before it calls into the game.
        #[allow(unsafe_code)]
        let status = unsafe { select(self.handle, slot) };
        ApiError::check(status)
    }
}
