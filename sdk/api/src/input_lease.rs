use amml_abi::{AmmlInputTable, AmmlMouseButton, AmmlMoveIntent};

use crate::{ApiError, ButtonHold, KeyCode, KeyHold, LeaseState, Look, MouseButton, MoveIntent};

/// An input lease: the right to inject input and to block real input. Dropping it releases everything
/// it holds, then ends it. Usable from any thread.
///
/// `Ok` from an injection means queued: the loader applies queued keys and buttons at the next
/// `FrameUpdate`. Injection from a suspended or lost lease gives `Refused`, before the input area's feed
/// is bound `NotReady`, and on a build without that area `NotAvailable`.
#[must_use = "dropping the lease ends it"]
pub struct InputLease {
    table: AmmlInputTable,
    id: u64,
}

impl InputLease {
    pub(crate) fn new(table: AmmlInputTable, id: u64) -> Self {
        Self { table, id }
    }

    /// The loader's id of the lease, as the lease events (`InputLeaseSuspended`, ...) carry it.
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn state(&self) -> LeaseState {
        // SAFETY: the loader accepts any lease id and gives `LOST` for an unknown one.
        #[allow(unsafe_code)]
        let state = unsafe { (self.table.lease_state)(self.id) };
        LeaseState::from(state)
    }

    /// Presses `key` until the returned hold is released.
    pub fn key_down(&self, key: KeyCode) -> Result<KeyHold, ApiError> {
        let mut hold = 0u64;
        // SAFETY: the loader accepts any lease id and key, and `hold` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.key_down)(self.id, key.0, &mut hold) };
        ApiError::check(status).map(|()| KeyHold::new(self.table.release_hold, hold))
    }

    /// Presses `button` until the returned hold is released.
    pub fn button_down(&self, button: MouseButton) -> Result<ButtonHold, ApiError> {
        let mut hold = 0u64;
        // SAFETY: the loader accepts any lease id and button, and `hold` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.button_down)(self.id, AmmlMouseButton::from(button), &mut hold) };
        ApiError::check(status).map(|()| ButtonHold::new(self.table.release_hold, hold))
    }

    /// Scrolls the wheel: one notch is 1.0, positive is away from the user. Whole notches reach the
    /// game; a fraction waits for the next call of this lease.
    pub fn wheel(&self, notches: f32) -> Result<(), ApiError> {
        // SAFETY: the loader accepts any lease id and value.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.wheel)(self.id, notches) };
        ApiError::check(status)
    }

    /// Turns the view; the turns of one frame are summed.
    pub fn turn(&self, delta: Look) -> Result<(), ApiError> {
        // SAFETY: the loader accepts any lease id and values.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.turn)(self.id, delta.yaw, delta.pitch) };
        ApiError::check(status)
    }

    /// Sets the movement until it is changed; `None` clears it. The last lease to set it wins; the
    /// other gets `InjectedInputReleased` with reason `REPLACED`.
    pub fn set_move(&self, intent: Option<MoveIntent>) -> Result<(), ApiError> {
        let intent = intent.map(AmmlMoveIntent::from);
        let pointer = intent.as_ref().map_or(std::ptr::null(), std::ptr::from_ref);
        // SAFETY: a non-null `pointer` points to the local `intent`, which lives for the call.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.set_move)(self.id, pointer) };
        ApiError::check(status)
    }

    /// Releases every hold and the movement of the lease. The lease stays. Always succeeds, also when
    /// the lease is suspended or lost.
    pub fn release_all(&self) {
        // SAFETY: the loader accepts any lease id.
        #[allow(unsafe_code)]
        unsafe {
            (self.table.release_all)(self.id)
        };
    }
}

impl Drop for InputLease {
    fn drop(&mut self) {
        self.release_all();
        // SAFETY: the loader accepts any lease id and gives `OK` for one already ended.
        #[allow(unsafe_code)]
        let _ = unsafe { (self.table.end_lease)(self.id) };
    }
}
