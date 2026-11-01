/// What a toast reported this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToastResponse {
    /// The close button was clicked. The caller removes the toast.
    pub dismissed: bool,
}
