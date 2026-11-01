use egui::Rect;

/// What a modal frame reported this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModalResponse {
    /// Escape, a click on the scrim, or the close button asked to close it. The caller closes it
    /// by not showing it again.
    pub closed: bool,
    /// The card's rect on the screen.
    pub card: Rect,
}
