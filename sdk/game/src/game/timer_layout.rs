use std::mem::offset_of;

/// Layout mirror of the game's `Timer` (128 bytes).
#[repr(C)]
pub struct TimerLayout {
    pub ticks_per_second: f32,            // 0x00 mTicksPerSecond
    pub ticks: i32,                       // 0x04 mTicks
    pub alpha: f32,                       // 0x08 mAlpha
    _time_scale: f32,                     // 0x0C mTimeScale
    pub passed_time: f32,                 // 0x10 mPassedTime
    _frame_step_alignment_remainder: f32, // 0x14 mFrameStepAlignmentRemainder
    _last_timestep: f32,                  // 0x18 mLastTimestep
    _overflow_time: f32,                  // 0x1C mOverflowTime
    _last_ms: i64,                        // 0x20 mLastMs
    _last_ms_sys_time: i64,               // 0x28 mLastMsSysTime
    _last_time_ms: i64,                   // 0x30 mLastTimeMs
    _adjust_time: f32,                    // 0x38 mAdjustTime
    _stepping_tick: f32,                  // 0x3C mSteppingTick
    _get_time_ms_callback: [u8; 0x40],    // 0x40 mGetTimeMSCallback (std::function)
}

const _: () = assert!(offset_of!(TimerLayout, ticks) == 0x04);
const _: () = assert!(offset_of!(TimerLayout, alpha) == 0x08);
const _: () = assert!(offset_of!(TimerLayout, passed_time) == 0x10);
const _: () = assert!(offset_of!(TimerLayout, _last_ms) == 0x20);
const _: () = assert!(offset_of!(TimerLayout, _stepping_tick) == 0x3C);
const _: () = assert!(offset_of!(TimerLayout, _get_time_ms_callback) == 0x40);
const _: () = assert!(size_of::<TimerLayout>() == 0x80);
