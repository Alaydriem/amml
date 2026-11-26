use amml_abi::WindowSize;

use crate::Size;

/// Typed reads and writes of a `WindowSize`.
pub trait WindowSizeExt {
    /// The client size the window got.
    fn size(&self) -> Size;
    /// Changes the client size the game sees; later listeners see the new value.
    fn set_size(&mut self, size: Size);
    fn minimised(&self) -> bool;
}

impl WindowSizeExt for WindowSize {
    fn size(&self) -> Size {
        Size { width: self.width, height: self.height }
    }

    fn set_size(&mut self, size: Size) {
        self.width = size.width;
        self.height = size.height;
    }

    fn minimised(&self) -> bool {
        self.minimised != 0
    }
}
