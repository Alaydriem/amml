use amml_abi::{AmmlTimeHold, AmmlTimeTable};

use crate::ApiError;

/// This mod's hold of the game clock (`Time::hold`). Dropping it releases the clock: the game's timers run on the wall
/// clock again from the next frame. A drop off the game thread is released at the next frame.
///
/// The loader also ends the hold by itself at a world leave, at this mod's disable, and when no `step_us` came for the
/// hold's watchdog time; `step_us` then gives `Err(Stale)`.
pub struct TimeHold {
    table: AmmlTimeTable,
    handle: AmmlTimeHold,
}

impl TimeHold {
    pub(crate) fn new(table: AmmlTimeTable, handle: AmmlTimeHold) -> Self {
        Self { table, handle }
    }

    /// Advances the held clock by exactly `micros` microseconds of game time: the game's timers take it at their next
    /// advance, and `TimeStepApplied` follows after that frame. Steps in one frame reach the timers together; every call,
    /// 0 included, feeds the watchdog. `Err(Stale)` once the hold ended; `Err(Refused)` off the game thread;
    /// `Err(Loader(INVALID_ARGUMENT))` for a negative `micros`.
    #[allow(unsafe_code)]
    pub fn step_us(&self, micros: i64) -> Result<(), ApiError> {
        // SAFETY: the table function is the loader's; the handle is a plain value.
        ApiError::check(unsafe { (self.table.step_us)(self.handle, micros) })
    }
}

impl Drop for TimeHold {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: as in `step_us`. A hold the loader already ended gives `Stale`, which a drop has no use for.
        let _ = unsafe { (self.table.release)(self.handle) };
    }
}
