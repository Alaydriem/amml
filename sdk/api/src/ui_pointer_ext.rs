use amml_abi::{AmmlInputMode, UiPointer};

use crate::{InputMode, Point, ScreenId, ViewId};

/// Typed reads of a `UiPointer`. Cancel it with `Cancel::cancel`: the game does not get that button event.
pub trait UiPointerExt {
    /// The screen of the view; `ScreenId::NONE` when the loader does not know it.
    fn screen(&self) -> ScreenId;

    /// The view that gets the button event. Every view of the screen stack gets the event, so a click is raised
    /// once for each of them.
    fn view(&self) -> ViewId;

    /// The index of the pointer button: 0 the primary button, 1 the second.
    fn pointer(&self) -> u8;

    /// Whether the button went down; false for an up.
    fn pressed(&self) -> bool;

    /// The last pointer position of the view, in GUI units.
    fn position(&self) -> Point;

    /// The device the game takes the pointer from; `Unknown` when the loader cannot read it.
    fn input_mode(&self) -> InputMode;
}

impl UiPointerExt for UiPointer {
    fn screen(&self) -> ScreenId {
        ScreenId(self.screen)
    }

    fn view(&self) -> ViewId {
        ViewId(self.view)
    }

    fn pointer(&self) -> u8 {
        self.pointer
    }

    fn pressed(&self) -> bool {
        self.pressed != 0
    }

    fn position(&self) -> Point {
        Point { x: self.x, y: self.y }
    }

    fn input_mode(&self) -> InputMode {
        InputMode::from(AmmlInputMode(self.input_mode))
    }
}
