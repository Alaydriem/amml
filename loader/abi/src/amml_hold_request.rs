/// What a mod asks for when it takes the game clock (`AmmlTimeTable::hold`).
///
/// `virtual_offset_ms` is added to the game's `steady_clock` count at the hold to give the virtual clock that the
/// clock-driven game systems see while the clock is held. `watchdog_ms` is how long the loader waits for a `step_us`
/// before it releases the hold; 0 takes the loader's default of 2000 ms.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlHoldRequest {
    pub virtual_offset_ms: i64,
    pub watchdog_ms: u32,
    pub _reserved: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlHoldRequest>() == 16);
