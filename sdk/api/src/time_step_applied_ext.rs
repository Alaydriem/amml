use amml_abi::TimeStepApplied;

/// Typed reads of a `TimeStepApplied`: a step of the held game clock reached the game's timers. Raised once, After the
/// frame in which the timers advanced by it, on the game thread.
pub trait TimeStepAppliedExt {
    /// The game time the held timers advanced since the hold, in microseconds.
    fn stepped_us(&self) -> i64;

    /// The ticks the game's sim timer handed out in that frame.
    fn sim_ticks(&self) -> i32;

    /// The alpha the game's sim timer handed out in that frame.
    fn sim_alpha(&self) -> f32;
}

impl TimeStepAppliedExt for TimeStepApplied {
    fn stepped_us(&self) -> i64 {
        self.stepped_us
    }

    fn sim_ticks(&self) -> i32 {
        self.sim_ticks
    }

    fn sim_alpha(&self) -> f32 {
        self.sim_alpha
    }
}
