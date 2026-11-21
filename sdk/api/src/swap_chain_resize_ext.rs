use amml_abi::SwapChainResize;

use crate::Size;

/// Typed reads and writes of a `SwapChainResize`.
pub trait SwapChainResizeExt {
    /// The size the game asked for; a side of 0 means the window's client size.
    fn size(&self) -> Size;
    /// Changes the size the buffers get; a side of 0 keeps what the game asked for.
    fn set_size(&mut self, size: Size);
}

impl SwapChainResizeExt for SwapChainResize {
    fn size(&self) -> Size {
        Size { width: self.width, height: self.height }
    }

    fn set_size(&mut self, size: Size) {
        self.width = size.width;
        self.height = size.height;
    }
}
