use amml_abi::{AmmlHoldRequest, AmmlStr, AmmlTimeHold, AmmlTimeState, AmmlTimeTable};

use crate::{ApiError, HoldRequest, TimeHold};

/// The game clock (`Ctx::time`, `Area::TIME_CONTROL`).
///
/// One mod at a time holds the clock: while held, the game's client timers hand out the ticks and the alpha of the game
/// time this mod steps with `TimeHold::step_us`, not of the wall clock, so the game advances by exactly the stepped time
/// whatever a frame costs. Without a time table (a loader before services v13) every read gives `None` and `hold` gives
/// `NotAvailable`.
pub struct Time {
    table: Option<AmmlTimeTable>,
    module: &'static str,
}

impl Time {
    pub(crate) fn new(table: Option<AmmlTimeTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// Takes the game clock. `Err(Refused)` while any mod holds it, this one included, and off the game thread;
    /// `Err(NotReady)` before a world is joined; `Err(NotAvailable)` without the area.
    #[allow(unsafe_code)]
    pub fn hold(&self, request: HoldRequest) -> Result<TimeHold, ApiError> {
        let Some(table) = self.table else { return Err(ApiError::NotAvailable) };
        let request = AmmlHoldRequest::from(request);
        let mut handle = AmmlTimeHold::default();
        // SAFETY: the table function is the loader's; the module id, the request and the out value live for the call.
        let status = unsafe { (table.hold)(AmmlStr::from_str(self.module), &raw const request, &raw mut handle) };
        ApiError::check(status)?;
        Ok(TimeHold::new(table, handle))
    }

    /// Whether a mod holds the game clock. `None` without the area.
    pub fn is_held(&self) -> Option<bool> {
        self.state().map(|state| state.held != 0)
    }

    /// The game time the held timers advanced since the hold, in microseconds. `None` without the area.
    pub fn stepped_us(&self) -> Option<i64> {
        self.state().map(|state| state.stepped_us)
    }

    /// The ticks the game's sim timer handed out at its last advance under the hold. `None` without the area.
    pub fn sim_ticks(&self) -> Option<i32> {
        self.state().map(|state| state.sim_ticks)
    }

    /// The alpha the game's sim timer handed out at its last advance under the hold. `None` without the area.
    pub fn sim_alpha(&self) -> Option<f32> {
        self.state().map(|state| state.sim_alpha)
    }

    /// The particle count of the last particle tick under the hold. `None` without the area.
    pub fn particle_count(&self) -> Option<u32> {
        self.state().map(|state| state.particle_count)
    }

    #[allow(unsafe_code)]
    fn state(&self) -> Option<AmmlTimeState> {
        let table = self.table?;
        let mut out = AmmlTimeState::default();
        // SAFETY: the table function is the loader's; `out` is a live local.
        let status = unsafe { (table.state)(&raw mut out) };
        status.is_ok().then_some(out)
    }
}
