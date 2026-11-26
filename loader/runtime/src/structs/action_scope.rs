use crate::BuildReentry;

/// One open mod action of `BuildReentry` on the thread that opened it; the action ends when the scope drops,
/// also during a panic unwind.
#[must_use = "the action ends when the scope is dropped"]
pub struct ActionScope<'r> {
    pub(crate) reentry: &'r BuildReentry,
}

impl Drop for ActionScope<'_> {
    fn drop(&mut self) {
        self.reentry.leave();
    }
}
