use crate::{Point, Size};

/// The game window a `FakeGame` serves after `set_window`: what the `Window` reads give. A `buffer`
/// with a side of 0 is a back buffer the loader does not know yet (`buffer_size` and `to_buffer` give
/// `None`). `focus` answers both `has_focus` and `is_foreground`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FakeWindow {
    pub client: Size,
    pub buffer: Size,
    pub focus: bool,
    pub dpi: u32,
    pub cursor: Point,
}

impl Default for FakeWindow {
    /// A focused 1280 x 720 window at 96 DPI whose back buffer has the client size, the cursor at 0,0.
    fn default() -> Self {
        let size = Size { width: 1280, height: 720 };
        Self { client: size, buffer: size, focus: true, dpi: 96, cursor: Point::default() }
    }
}
