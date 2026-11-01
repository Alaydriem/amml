#![allow(unsafe_code)]

use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{ModRuntime, PanicText};

/// The state behind one detour made by `amml_detour!`: the `original` slot the loader fills and the
/// panic guard around the detour body.
///
/// A body that panics is logged once, the whole mod is marked disabled, and the original runs for
/// that call, unless the body already called it (`amml_detour!` keeps that result). From then on
/// every detour of the mod calls its original directly and skips the body.
pub struct DetourGuard {
    label: &'static str,
    original: AtomicPtr<c_void>,
}

impl DetourGuard {
    pub const fn new(label: &'static str) -> Self {
        Self { label, original: AtomicPtr::new(std::ptr::null_mut()) }
    }

    /// The slot to give to `Api::hook`.
    pub fn original(&'static self) -> &'static AtomicPtr<c_void> {
        &self.original
    }

    /// The next function in the chain, read from the slot.
    ///
    /// A null slot means the detour ran before the loader installed the hook, which cannot happen
    /// with a correct loader; the process ends because there is nothing to return.
    pub fn original_fn<F: Copy>(&self) -> F {
        let next = self.original.load(Ordering::Acquire);
        if next.is_null() || std::mem::size_of::<F>() != std::mem::size_of::<*mut c_void>() {
            if let Some(runtime) = ModRuntime::get() {
                runtime.logger().error(&format!("detour {} ran without an original function", self.label));
            }
            std::process::abort();
        }
        // SAFETY: `F` is a function pointer type of the hooked function's signature (the macro
        // fixes it), the size matches a pointer, and the loader stored the next hook or the
        // trampoline, which have that signature, before the detour could run.
        unsafe { std::mem::transmute_copy::<*mut c_void, F>(&next) }
    }

    /// Runs `body` unless the mod is disabled; on a panic or when disabled, runs `original`.
    pub fn run<R>(&self, body: impl FnOnce() -> R, original: impl FnOnce() -> R) -> R {
        if ModRuntime::is_disabled() {
            return original();
        }
        match catch_unwind(AssertUnwindSafe(body)) {
            Ok(value) => value,
            Err(payload) => {
                if ModRuntime::disable()
                    && let Some(runtime) = ModRuntime::get()
                {
                    let message = format!(
                        "detour {} panicked: {}; the mod is disabled and calls the original",
                        self.label,
                        PanicText::of(payload.as_ref())
                    );
                    runtime.logger().error(&message);
                }
                original()
            }
        }
    }
}
