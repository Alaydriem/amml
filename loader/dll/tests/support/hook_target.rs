use std::hint::black_box;

/// A function of the test binary that a mod hooks through the loader.
pub struct HookTarget;

impl HookTarget {
    #[inline(never)]
    pub extern "C" fn run(x: i32) -> i32 {
        black_box(x) + black_box(1)
    }

    /// The address of `run`.
    pub fn address() -> u64 {
        black_box(Self::run as extern "C" fn(i32) -> i32) as usize as u64
    }

    /// Calls `run` through its address, so the call goes through whatever the loader patched in.
    pub fn call(x: i32) -> i32 {
        black_box(Self::run as extern "C" fn(i32) -> i32)(black_box(x))
    }
}
