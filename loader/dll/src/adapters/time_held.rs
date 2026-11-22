use amml_abi::AmmlTimeHold;
use amml_runtime::{TimeHoldBook, VirtualClock};

/// What the game clock holds under its lock: the hold, the stepped game time, the four timers of the hold (game sim,
/// game real, client sim, client real), whether a step reached the timers since the last `TimeStepApplied`, what the
/// game sim timer handed out last, the releases queued from other threads, the stepped game time at the last particle
/// tick and the particle count of that tick.
#[derive(Default)]
pub struct TimeHeld {
    pub book: TimeHoldBook,
    pub clock: VirtualClock,
    pub timers: [usize; VirtualClock::TIMERS],
    pub applied: bool,
    pub sim_ticks: i32,
    pub sim_alpha: f32,
    pub queued: Vec<AmmlTimeHold>,
    pub particle_stepped_us: i64,
    pub particle_count: u32,
}

impl TimeHeld {
    pub fn new() -> Self {
        Self::default()
    }

    /// The hold ended: the clock and the timers of the hold are forgotten. The book is left as the caller made it.
    pub fn reset(&mut self) {
        self.clock = VirtualClock::new();
        self.timers = [0; VirtualClock::TIMERS];
        self.applied = false;
        self.sim_ticks = 0;
        self.sim_alpha = 0.0;
        self.queued.clear();
        self.particle_stepped_us = 0;
        self.particle_count = 0;
    }
}
