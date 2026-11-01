use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;

use amml_sdk::{ModRuntime, PanicText};

use crate::structs::StopLatch;

/// The body of the client tick detour, kept apart from the hook so a test can drive it.
pub struct ClientUpdate;

impl ClientUpdate {
    /// Calls `original` with the arguments unchanged, then runs `body` with the client instance
    /// when `latch` is lowered, the mod is not disabled and `this` is not null.
    ///
    /// A panic in `body` is logged and raises `latch`; the value of `original` is returned in all
    /// cases. Once the latch is raised, `body` does not run again, so the panic is logged once. The
    /// state a panicked `body` left half done is never used again for that reason.
    ///
    /// Two off switches apply: the caller's `latch`, and the mod-wide flag that a panic in an
    /// `amml_detour!` detour sets (`ModRuntime::is_disabled`). Either one skips `body`; the original
    /// still runs.
    ///
    /// The detour that calls `run` must be `extern "C"` and must not unwind. A panic in `original`
    /// is not caught, because it is a game call.
    ///
    /// # Example
    ///
    /// The whole mod-side pattern: a static guard that holds the original slot, the detour in the
    /// `ClientUpdateFn` shape, and the original read through `DetourGuard::original_fn`, which
    /// ends the process on a null slot instead of calling it.
    ///
    /// ```no_run
    /// use std::ffi::c_void;
    ///
    /// use amml_game::hooks::{ClientUpdate, ClientUpdateFn};
    /// use amml_game::{HeldClient, StopLatch};
    /// use amml_sdk::DetourGuard;
    ///
    /// static GUARD: DetourGuard = DetourGuard::new("client_update");
    /// static LATCH: StopLatch = StopLatch::new();
    /// static HELD: HeldClient = HeldClient::new();
    ///
    /// extern "C" fn detour(this: usize, init_finished: u8, extra: u64) -> u8 {
    ///     let original: ClientUpdateFn = GUARD.original_fn();
    ///     ClientUpdate::run(
    ///         // SAFETY: `original` is the next function in the chain, with the hooked signature.
    ///         |this, init_finished, extra| unsafe { original(this, init_finished, extra) },
    ///         this,
    ///         init_finished,
    ///         extra,
    ///         &LATCH,
    ///         |client| HELD.set(client.as_ptr()),
    ///     )
    /// }
    ///
    /// // The detour has the shape the hook expects.
    /// const _: extern "C" fn(usize, u8, u64) -> u8 = detour;
    /// // `GUARD.original()` is the slot to give to `Api::hook`.
    /// let _slot: &'static std::sync::atomic::AtomicPtr<c_void> = GUARD.original();
    /// ```
    pub fn run(
        original: impl FnOnce(usize, u8, u64) -> u8,
        this: usize,
        init_finished: u8,
        extra: u64,
        latch: &StopLatch,
        body: impl FnOnce(NonNull<c_void>),
    ) -> u8 {
        let result = original(this, init_finished, extra);
        if latch.raised() || ModRuntime::is_disabled() {
            return result;
        }
        let Some(client) = NonNull::new(this as *mut c_void) else {
            return result;
        };
        if let Err(payload) = catch_unwind(AssertUnwindSafe(|| body(client))) {
            latch.raise();
            if let Some(runtime) = ModRuntime::get() {
                runtime.logger().error(&format!(
                    "client update body panicked: {}; the mod stops its per-tick work",
                    PanicText::of(payload.as_ref())
                ));
            }
        }
        result
    }
}
