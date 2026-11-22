/// The state of the game clock (`AmmlTimeTable::state`).
///
/// `held` is 1 while a mod holds the clock. `stepped_us` is the game time the held timers advanced since the hold, in
/// microseconds; `sim_ticks` and `sim_alpha` are the ticks and the alpha the game's sim timer handed out at its last
/// advance under the hold. `particle_count` is the particle count of the last particle tick under the hold, 0 until
/// the loader serves the particle clock.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AmmlTimeState {
    pub held: u8,
    pub _reserved: [u8; 3],
    pub sim_ticks: i32,
    pub stepped_us: i64,
    pub sim_alpha: f32,
    pub particle_count: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlTimeState>() == 24);
