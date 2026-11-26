use std::cell::RefCell;
use std::time::Instant;

use amml_abi::{AmmlHoldRequest, AmmlStatus, AmmlTimeHold, AmmlTimeState};
use amml_runtime::{TimeHoldBook, TimeReleaseReason, VirtualClock};

/// The game clock side of a `FakeGame`: the hold of the fake mod, the stepped game time and the writes that went out,
/// one line each (`hold offset=<ms>`, `step <micros>`, `release reason=<reason>`). A step reaches the fake's timers at
/// once. The fake has one mod, so a second hold is always its own and is refused.
#[derive(Default)]
pub(crate) struct FakeTime {
    book: RefCell<TimeHoldBook>,
    clock: RefCell<VirtualClock>,
    writes: RefCell<Vec<String>>,
}

impl FakeTime {
    // The holder of every hold: the fake has one mod.
    const OWNER: usize = 1;

    pub(crate) fn writes(&self) -> Vec<String> {
        self.writes.borrow().clone()
    }

    pub(crate) fn hold(&self, request: AmmlHoldRequest) -> Result<AmmlTimeHold, AmmlStatus> {
        let hold = self
            .book
            .borrow_mut()
            .hold(Self::OWNER, request.watchdog_ms, Instant::now())
            .map_err(|_| AmmlStatus::REFUSED)?;
        *self.clock.borrow_mut() = VirtualClock::new();
        self.writes.borrow_mut().push(format!("hold offset={}", request.virtual_offset_ms));
        Ok(hold)
    }

    pub(crate) fn step_us(&self, hold: AmmlTimeHold, micros: i64) -> Result<(), AmmlStatus> {
        if micros < 0 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        self.book.borrow_mut().step(hold, Instant::now()).map_err(|_| AmmlStatus::STALE)?;
        let mut clock = self.clock.borrow_mut();
        clock.step_us(micros);
        clock.apply();
        self.writes.borrow_mut().push(format!("step {micros}"));
        Ok(())
    }

    pub(crate) fn release(&self, hold: AmmlTimeHold) -> Result<(), AmmlStatus> {
        self.book.borrow_mut().release(hold).map_err(|_| AmmlStatus::STALE)?;
        self.ended(TimeReleaseReason::Release);
        Ok(())
    }

    pub(crate) fn state(&self) -> AmmlTimeState {
        AmmlTimeState {
            held: u8::from(self.book.borrow().holder().is_some()),
            stepped_us: self.clock.borrow().stepped_us(),
            ..AmmlTimeState::default()
        }
    }

    /// The fake mod is disabled: its hold ends.
    pub(crate) fn disable_owner(&self) {
        if self.book.borrow_mut().disable(Self::OWNER) {
            self.ended(TimeReleaseReason::Disabled);
        }
    }

    fn ended(&self, reason: TimeReleaseReason) {
        *self.clock.borrow_mut() = VirtualClock::new();
        self.writes.borrow_mut().push(format!("release reason={}", reason.name()));
    }
}
