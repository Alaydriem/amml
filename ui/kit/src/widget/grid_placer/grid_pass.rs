use egui::Id;

use super::{GridCellRef, GridPlacerResponse};

/// What one frame of a grid placer carries from part to part.
pub(crate) struct GridPass {
    pub held: Option<u64>,
    pub focus: GridCellRef,
    pub dragged: Option<u64>,
    /// The widget that had the keyboard focus when the frame began; only it reads the keys, so a
    /// focus moved during the frame does not read the same key again.
    pub keyboard: Option<Id>,
    /// A chip was picked up this frame, so its Enter does not also place it.
    pub picked: bool,
    pub out: GridPlacerResponse,
}
