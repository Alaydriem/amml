use std::hash::{Hash, Hasher};

mod id_scope;

pub use id_scope::IdScope;

/// The egui ids amml-ui keeps for itself, or adds under a caller's id. Widgets that keep state take
/// their base id from the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmmlUiId {
    /// The localizer the widgets resolve `Text` with.
    Locale,
    /// A segmented control's selection fill, as it slides.
    SegmentSlide,
    /// A tab row underline's two edges, as they slide.
    UnderlineLeft,
    UnderlineRight,
    /// A field's box, for its hover.
    FieldBox,
    /// A text field's value when it took the focus, restored on Escape.
    FieldStart,
    /// A text field's clear button.
    FieldClear,
    /// The typed text of a drag or colour field, and its text edit.
    InlineBuffer,
    InlineEdit,
    /// Set when typing starts; the edit takes the focus on its first frame.
    InlineFocus,
    /// Why a drag or colour field's last typed value was refused.
    FieldError,
    /// The option the keyboard has reached in an open dropdown list.
    DropdownHighlight,
    /// The confirm popover's area.
    ConfirmPopover,
    /// A grid placer's piece chip, its cells, the unplace button of a filled cell and the tray drop zone.
    GridChip,
    GridCell,
    GridUnplace,
    GridTray,
    /// The piece a grid placer holds for the keyboard route, and the cell the keyboard has reached.
    GridHeld,
    GridFocusCell,
    /// The layer the dragged chip floats on.
    GridDrag,
}

impl Hash for AmmlUiId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        IdScope::hash_type::<Self, H>(state);
        std::mem::discriminant(self).hash(state);
    }
}
