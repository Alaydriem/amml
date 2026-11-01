use std::panic::{AssertUnwindSafe, catch_unwind};

use amml_abi::AmmlStatus;

/// Keeps a panic from unwinding across the C ABI.
pub struct Boundary;

impl Boundary {
    /// Runs `body`. A panic is logged and gives `fallback`.
    pub fn run<R>(fallback: R, body: impl FnOnce() -> R) -> R {
        match catch_unwind(AssertUnwindSafe(body)) {
            Ok(value) => value,
            Err(payload) => {
                let message = Self::panic_text(payload.as_ref());
                curia::error!("a panic was stopped at the C boundary", { message: message });
                fallback
            }
        }
    }

    /// Runs `body`; a panic gives `AmmlStatus::PANICKED`.
    pub fn guard(body: impl FnOnce() -> AmmlStatus) -> AmmlStatus {
        Self::run(AmmlStatus::PANICKED, body)
    }

    fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
        if let Some(text) = payload.downcast_ref::<&str>() {
            (*text).to_string()
        } else if let Some(text) = payload.downcast_ref::<String>() {
            text.clone()
        } else {
            "panic without a message".to_string()
        }
    }
}
