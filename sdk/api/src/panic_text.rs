use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Turns a panic payload into text for a log line.
pub struct PanicText;

impl PanicText {
    pub fn of(payload: &(dyn Any + Send)) -> String {
        if let Some(text) = payload.downcast_ref::<&str>() {
            (*text).to_string()
        } else if let Some(text) = payload.downcast_ref::<String>() {
            text.clone()
        } else {
            "panic without a message".to_string()
        }
    }

    /// Gives the text of `payload` to `report`, then drops the payload, all inside `catch_unwind`.
    /// A panic from `report` or from the payload's `Drop` is caught, and its own payload is leaked,
    /// so nothing unwinds out of this call.
    pub fn consume(payload: Box<dyn Any + Send>, report: impl FnOnce(&str)) {
        let outcome = catch_unwind(AssertUnwindSafe(move || {
            let text = Self::of(payload.as_ref());
            report(&text);
            drop(payload);
        }));
        if let Err(nested) = outcome {
            std::mem::forget(nested);
        }
    }
}
