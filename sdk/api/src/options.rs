use amml_abi::{AmmlOptionsTable, AmmlStr};

use crate::{ApiError, Client, OptionKey, Persistence};

/// The game options (`Ctx::options`): the view distance and a few integer options.
///
/// `view_distance_chunks` needs `Area::OPTIONS_VIEW_DISTANCE`; the integer options need `Area::OPTIONS_INT`. Game thread
/// only. Without an options table (a loader before services v12) every read gives `None` and every write gives
/// `NotAvailable`.
pub struct Options {
    table: Option<AmmlOptionsTable>,
    module: &'static str,
}

impl Options {
    pub(crate) fn new(table: Option<AmmlOptionsTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// The view distance of `client` in chunks. `None` off the game thread, for a client that is not the primary client,
    /// before the first frame and without the area.
    #[allow(unsafe_code)]
    pub fn view_distance_chunks(&self, client: &Client<'_>) -> Option<u32> {
        let table = self.table?;
        let mut out = 0u32;
        // SAFETY: the table function is the loader's, which checks the handle before it reads game memory; `out` is a
        // live local.
        let status = unsafe { (table.view_distance_chunks)(client.handle(), &raw mut out) };
        status.is_ok().then_some(out)
    }

    /// The value of the integer option `key`. `None` off the game thread, before the first frame, without the area and
    /// on a build where the loader cannot find the option.
    #[allow(unsafe_code)]
    pub fn get_int(&self, key: OptionKey) -> Option<i32> {
        let table = self.table?;
        let mut out = 0i32;
        // SAFETY: the table function is the loader's; `out` is a live local.
        let status = unsafe { (table.get_int)(key.to_abi(), &raw mut out) };
        status.is_ok().then_some(out)
    }

    /// Sets the integer option `key` to `value`. With `Persistence::Transient` the game does not save the change, and
    /// the loader puts the value from before the first transient set back when you call `release_transient` or when this
    /// mod is disabled, unless the user changed the option meanwhile. `Err(Refused)` off the game thread and when
    /// another mod holds a transient value of the key; `Err(NotReady)` before the first frame; `Err(NotAvailable)`
    /// without the area or on a build where the loader cannot find the option.
    #[allow(unsafe_code)]
    pub fn set_int(&self, key: OptionKey, value: i32, persistence: Persistence) -> Result<(), ApiError> {
        let Some(table) = self.table else { return Err(ApiError::NotAvailable) };
        // SAFETY: the table function is the loader's; the module id lives for the call.
        let status =
            unsafe { (table.set_int)(AmmlStr::from_str(self.module), key.to_abi(), value, persistence.to_abi()) };
        ApiError::check(status)
    }

    /// Ends the transient value of `key` that this mod holds: the value from before its first transient set is put back
    /// and saved when the option still holds the value this mod set, and left alone when the user changed it. `Ok` with
    /// nothing written when no mod holds a transient value of the key. `Err(Refused)` when another mod holds it.
    #[allow(unsafe_code)]
    pub fn release_transient(&self, key: OptionKey) -> Result<(), ApiError> {
        let Some(table) = self.table else { return Err(ApiError::NotAvailable) };
        // SAFETY: the table function is the loader's; the module id lives for the call.
        let status = unsafe { (table.release_transient)(AmmlStr::from_str(self.module), key.to_abi()) };
        ApiError::check(status)
    }
}
