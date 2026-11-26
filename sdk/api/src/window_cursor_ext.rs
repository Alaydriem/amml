use amml_abi::{AmmlCursorShape, WindowCursor};

use crate::CursorShape;

/// Writes of a `WindowCursor`.
pub trait WindowCursorExt {
    /// Sets `shape` over the game window for this message; the game does not set its own.
    fn set_shape(&mut self, shape: CursorShape);
    /// Leaves the shape to the game again, for a shape an earlier listener set.
    fn clear_shape(&mut self);
}

impl WindowCursorExt for WindowCursor {
    fn set_shape(&mut self, shape: CursorShape) {
        self.shape = AmmlCursorShape::from(shape);
    }

    fn clear_shape(&mut self) {
        self.shape = AmmlCursorShape::NONE;
    }
}
