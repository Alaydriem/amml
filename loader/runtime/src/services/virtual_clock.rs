use crate::TimerReading;

/// The game time of a held clock, in whole microseconds from the tick boundary of the hold.
///
/// `step_us` queues game time; `apply` moves what is queued to the timers' time once per frame (the first held timer
/// advance of the frame calls it), so every timer of a frame sees the same game time. `read` gives one timer its ticks
/// since its last read and its alpha, from the integer microseconds: the ticks handed out to a timer always sum to the
/// whole ticks of the stepped time, and one `step_us(n)` is exactly `n` microseconds of game time, with no drift from
/// a float tick count. A timer's rate (ticks per second) is taken in thousandths of a tick per second.
#[derive(Debug, Default)]
pub struct VirtualClock {
    stepped_us: i64,
    pending_us: i64,
    given: [i64; VirtualClock::TIMERS],
}

impl VirtualClock {
    /// The timers a clock keeps a tick count for.
    pub const TIMERS: usize = 4;
    // The rate of a timer whose own rate is not a positive number: the game's 20 ticks per second.
    const DEFAULT_MILLI_TPS: i128 = 20_000;
    const MICROS_PER_SECOND: i128 = 1_000_000;
    const MILLI: i128 = 1000;

    /// A clock at the tick boundary of a hold, with nothing stepped.
    pub fn new() -> Self {
        Self { stepped_us: 0, pending_us: 0, given: [0; Self::TIMERS] }
    }

    /// Queues `micros` of game time for the next `apply`. A negative value is taken as 0; the loader refuses it before.
    pub fn step_us(&mut self, micros: i64) {
        self.pending_us = self.pending_us.saturating_add(micros.max(0));
    }

    /// Moves the queued game time to the timers' time. True when there was any.
    pub fn apply(&mut self) -> bool {
        if self.pending_us == 0 {
            return false;
        }
        self.stepped_us = self.stepped_us.saturating_add(self.pending_us);
        self.pending_us = 0;
        true
    }

    /// The game time the timers' time advanced since the hold.
    pub fn stepped_us(&self) -> i64 {
        self.stepped_us
    }

    /// What the timer `timer` (below `TIMERS`) hands out now at `ticks_per_second`: the whole ticks of the stepped time
    /// it has not handed out yet (never below 0), and the alpha. A timer index out of range gets the alpha and no tick.
    pub fn read(&mut self, timer: usize, ticks_per_second: f32) -> TimerReading {
        let scaled = i128::from(self.stepped_us) * Self::milli_tps(ticks_per_second);
        let per_tick = Self::MICROS_PER_SECOND * Self::MILLI;
        let whole = i64::try_from(scaled / per_tick).unwrap_or(i64::MAX);
        let alpha = ((scaled % per_tick) as f64 / per_tick as f64) as f32;
        let Some(given) = self.given.get_mut(timer) else {
            return TimerReading { ticks: 0, alpha };
        };
        // After a rate drop the whole ticks at the new rate can be below what the timer handed out: it hands out none
        // (the game never writes a negative tick count) and counts on from the new rate.
        let ticks = i32::try_from(whole.saturating_sub(*given).max(0)).unwrap_or(i32::MAX);
        *given = whole;
        TimerReading { ticks, alpha }
    }

    // The rate in thousandths of a tick per second.
    fn milli_tps(ticks_per_second: f32) -> i128 {
        if !ticks_per_second.is_finite() || ticks_per_second <= 0.0 {
            return Self::DEFAULT_MILLI_TPS;
        }
        (f64::from(ticks_per_second) * Self::MILLI as f64).round() as i128
    }
}
