/// Keeps a panic from unwinding into C++ frames.
pub struct CallbackGuard;

impl CallbackGuard {
    /// Runs `body`. A panic aborts the process, because a callback that C++ called has no way to report
    /// failure and unwinding through C++ frames is undefined behaviour.
    pub fn run<R>(body: impl FnOnce() -> R) -> R {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)) {
            Ok(value) => value,
            Err(_) => std::process::abort(),
        }
    }
}
