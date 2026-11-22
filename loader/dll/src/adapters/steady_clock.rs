use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

/// The game's `std::chrono::steady_clock`: the performance counter scaled to nanoseconds as MSVC's `steady_clock::now`
/// scales it (survey section 8, row T7: the game calls `_Query_perf_frequency` and `_Query_perf_counter`). Whole seconds
/// and the rest are scaled apart, so the value equals the game's own count.
pub struct SteadyClock;

impl SteadyClock {
    const NANOS_PER_SECOND: i128 = 1_000_000_000;

    /// The steady clock count now, in nanoseconds; 0 when the counter cannot be read.
    pub fn now_ns() -> i64 {
        let (mut frequency, mut counter) = (0i64, 0i64);
        // SAFETY: both calls only write the one integer each is given.
        let read = unsafe {
            QueryPerformanceFrequency(&raw mut frequency) != 0 && QueryPerformanceCounter(&raw mut counter) != 0
        };
        if !read || frequency <= 0 {
            return 0;
        }
        let (frequency, counter) = (i128::from(frequency), i128::from(counter));
        let whole = counter / frequency * Self::NANOS_PER_SECOND;
        let part = counter % frequency * Self::NANOS_PER_SECOND / frequency;
        i64::try_from(whole + part).unwrap_or(i64::MAX)
    }
}
