use crate::{AmmlEvent, AmmlEventHeader};

/// A step of the held game clock reached the timers: raised once, After the frame (`MinecraftGame::update`) in which
/// the timers advanced by it, on the game thread.
///
/// `stepped_us` is the game time the held timers advanced since the hold, in microseconds; `sim_ticks` and `sim_alpha`
/// are the ticks and the alpha the game's sim timer handed out in that frame.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TimeStepApplied {
    pub header: AmmlEventHeader,
    pub stepped_us: i64,
    pub sim_ticks: i32,
    pub sim_alpha: f32,
}

impl AmmlEvent for TimeStepApplied {
    const ID: &'static str = "time.step_applied";
}

const _: () = assert!(core::mem::size_of::<TimeStepApplied>() == 32);
