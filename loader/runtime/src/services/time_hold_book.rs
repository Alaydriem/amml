use std::time::{Duration, Instant};

use amml_abi::AmmlTimeHold;

use crate::TimeHoldRefusal;

/// Which mod holds the game clock, and when the hold was last fed.
///
/// One hold for the process: any `hold` while one exists is `Taken`, the holder's too. Each hold has a new generation,
/// so a handle of an ended hold is `Stale`. A hold ends by its `release`, a world leave, the holder's disable, or the
/// watchdog: no `step` for its watchdog time (checked by the loader at each frame). The book makes no game call and
/// reads no clock: the caller passes the time.
#[derive(Debug, Default)]
pub struct TimeHoldBook {
    // The holder, its handle, its watchdog time and when it was last fed.
    held: Option<(usize, AmmlTimeHold, Duration, Instant)>,
    generation: u32,
}

impl TimeHoldBook {
    /// The watchdog time of a hold that asks for 0.
    pub const DEFAULT_WATCHDOG: Duration = Duration::from_millis(2000);
    /// The `id` of every hold: the one clock of the process.
    pub const CLOCK_ID: u32 = 1;

    pub fn new() -> Self {
        Self { held: None, generation: 0 }
    }

    /// The mod that holds the clock.
    pub fn holder(&self) -> Option<usize> {
        self.held.map(|(owner, ..)| owner)
    }

    /// The mod `owner` takes the clock at `now`, released after `watchdog_ms` without a step (0: `DEFAULT_WATCHDOG`).
    /// `Taken` while any hold exists.
    pub fn hold(&mut self, owner: usize, watchdog_ms: u32, now: Instant) -> Result<AmmlTimeHold, TimeHoldRefusal> {
        if self.held.is_some() {
            return Err(TimeHoldRefusal::Taken);
        }
        self.generation = self.generation.wrapping_add(1).max(1);
        let hold = AmmlTimeHold { id: Self::CLOCK_ID, generation: self.generation };
        let watchdog =
            if watchdog_ms == 0 { Self::DEFAULT_WATCHDOG } else { Duration::from_millis(u64::from(watchdog_ms)) };
        self.held = Some((owner, hold, watchdog, now));
        Ok(hold)
    }

    /// A step of `hold` at `now`: feeds the watchdog. `Stale` for a hold that is not the current one.
    pub fn step(&mut self, hold: AmmlTimeHold, now: Instant) -> Result<(), TimeHoldRefusal> {
        match self.held.as_mut() {
            Some((_, current, _, fed)) if *current == hold => {
                *fed = now;
                Ok(())
            }
            _ => Err(TimeHoldRefusal::Stale),
        }
    }

    /// The release of `hold`. `Stale` for a hold that is not the current one.
    pub fn release(&mut self, hold: AmmlTimeHold) -> Result<(), TimeHoldRefusal> {
        match self.held {
            Some((_, current, ..)) if current == hold => {
                self.held = None;
                Ok(())
            }
            _ => Err(TimeHoldRefusal::Stale),
        }
    }

    /// The mod `owner` is disabled: true when it held the clock (the hold ended).
    pub fn disable(&mut self, owner: usize) -> bool {
        if self.holder() != Some(owner) {
            return false;
        }
        self.held = None;
        true
    }

    /// Before `WorldLeft`: the hold ends. Gives the mod that held the clock.
    pub fn world_left(&mut self) -> Option<usize> {
        self.held.take().map(|(owner, ..)| owner)
    }

    /// The watchdog look at `now`: a hold not fed for longer than its watchdog time ends. Gives its mod.
    pub fn watchdog(&mut self, now: Instant) -> Option<usize> {
        let (owner, _, watchdog, fed) = self.held?;
        if now.saturating_duration_since(fed) <= watchdog {
            return None;
        }
        self.held = None;
        Some(owner)
    }
}
