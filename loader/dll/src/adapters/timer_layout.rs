use std::mem::offset_of;

use amml_runtime::TimerReading;

/// Layout mirror of the prefix of the game's `Timer` that holds on every supported build (survey section 8, row T2:
/// `Timer::advanceTime` writes ticks at 4, alpha at 8 and passed time at 10h, and reads the rate at 0 and the
/// overflow time at 1Ch, on 26.63 and 70.26). The size and the tail differ (80h on 26.63, 70h on 70.26, row T3), so
/// the mirror ends at 20h and the loader never reads past it.
#[repr(C)]
pub struct TimerLayout {
    pub ticks_per_second: f32,            // 0x00 mTicksPerSecond
    pub ticks: i32,                       // 0x04 mTicks
    pub alpha: f32,                       // 0x08 mAlpha
    _time_scale: f32,                     // 0x0C mTimeScale
    pub passed_time: f32,                 // 0x10 mPassedTime
    _frame_step_alignment_remainder: f32, // 0x14 mFrameStepAlignmentRemainder
    _last_timestep: f32,                  // 0x18 mLastTimestep
    pub overflow_time: f32,               // 0x1C mOverflowTime
}

const _: () = assert!(offset_of!(TimerLayout, ticks) == 0x04);
const _: () = assert!(offset_of!(TimerLayout, alpha) == 0x08);
const _: () = assert!(offset_of!(TimerLayout, passed_time) == 0x10);
const _: () = assert!(offset_of!(TimerLayout, overflow_time) == 0x1C);
const _: () = assert!(size_of::<TimerLayout>() == 0x20);

impl TimerLayout {
    /// The rate of the timer at `timer`.
    ///
    /// # Safety
    /// `timer` is the address of a live game `Timer`.
    pub unsafe fn ticks_per_second(timer: usize) -> f32 {
        let timer = std::ptr::with_exposed_provenance::<TimerLayout>(timer);
        // SAFETY: the caller guarantees a live Timer, whose prefix the mirror covers.
        unsafe { (&raw const (*timer).ticks_per_second).read() }
    }

    /// The overflow time of the timer at `timer`.
    ///
    /// # Safety
    /// As for `ticks_per_second`.
    pub unsafe fn overflow_time(timer: usize) -> f32 {
        let timer = std::ptr::with_exposed_provenance::<TimerLayout>(timer);
        // SAFETY: as in `ticks_per_second`.
        unsafe { (&raw const (*timer).overflow_time).read() }
    }

    /// Writes the held clock's `reading` into the timer at `timer` (ticks, alpha, and the alpha as the passed time),
    /// and puts the overflow time back to `overflow`, the value from before the game's advance.
    ///
    /// # Safety
    /// `timer` is the address of a live game `Timer` that no other thread writes now.
    pub unsafe fn write_held(timer: usize, reading: TimerReading, overflow: f32) {
        let timer = std::ptr::with_exposed_provenance_mut::<TimerLayout>(timer);
        // SAFETY: the caller guarantees a live Timer that only this thread writes; the mirror covers the four fields.
        unsafe {
            (&raw mut (*timer).ticks).write(reading.ticks);
            (&raw mut (*timer).alpha).write(reading.alpha);
            (&raw mut (*timer).passed_time).write(reading.alpha);
            (&raw mut (*timer).overflow_time).write(overflow);
        }
    }
}
