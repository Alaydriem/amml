use std::any::Any;

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
}
