use amml_abi::AmmlHoldRequest;

/// What `Time::hold` asks for.
///
/// `virtual_offset_ms` is added to the game's `steady_clock` count at the hold to give the virtual clock that the
/// clock-driven game systems see while the clock is held. `watchdog_ms` is how long the loader waits for a
/// `TimeHold::step_us` before it releases the hold by itself; 0 takes the loader's default of 2000 ms.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoldRequest {
    pub virtual_offset_ms: i64,
    pub watchdog_ms: u32,
}

impl From<HoldRequest> for AmmlHoldRequest {
    fn from(request: HoldRequest) -> Self {
        Self { virtual_offset_ms: request.virtual_offset_ms, watchdog_ms: request.watchdog_ms, _reserved: 0 }
    }
}
