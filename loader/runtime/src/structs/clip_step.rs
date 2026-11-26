use super::ClipKind;

/// What the window must do with the system cursor clip after a `ClipBook` change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipStep {
    /// Set the clip of this kind, at the current client centre.
    Apply(ClipKind),
    /// Give the clip back; the loader set it and nothing wants it now.
    Release,
    /// The clip stays as it is.
    Keep,
}
