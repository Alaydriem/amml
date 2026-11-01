use egui::Pos2;

use super::{EasingFamily, HandleIndex};

/// What the easing popover asks the host to do. Preset indexes are into `EasingPopoverView::presets`, saved
/// indexes into `EasingPopoverView::saved`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingPopoverAction {
    PickPreset(usize),
    PickSaved(usize),
    /// A right-click on a saved preset: its menu at the pointer (Plan 08).
    OpenSavedMenu(usize, Pos2),
    /// A handle dragged to this point of the unit box; x stays in 0 to 1, y may overshoot.
    MoveHandle(HandleIndex, Pos2),
    SetFamily(EasingFamily),
    SavePreset,
    /// Turn the named curve into a custom bezier with handles.
    MakeCustom,
    Close,
}
