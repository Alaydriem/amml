use std::ffi::c_void;

use amml_msvc::SharedPtr;
use amml_runtime::ShapeBook;

/// What the shapes hold under their lock: the shapes of each mod and their game ids, the `PrimitiveShapesPacket` the
/// loader made in this world (one use of the game's control block), and the game-heap block of `capacity` payloads that
/// backs the packet's shape vector. The block grows to the largest batch; between deliveries the vector is empty
/// (first = last) and still owns the block, so the game frees it with the packet.
pub struct ShapesHeld {
    pub book: ShapeBook,
    pub packet: Option<SharedPtr<c_void>>,
    pub block: *mut u8,
    pub capacity: usize,
}

// SAFETY: the packet and its block are used only under the adapter's lock, on the thread that holds it; the game's
// control block counts its uses with atomics, so the last use may be given back on any thread.
unsafe impl Send for ShapesHeld {}

impl ShapesHeld {
    pub fn new() -> Self {
        Self { book: ShapeBook::new(), packet: None, block: std::ptr::null_mut(), capacity: 0 }
    }

    /// Gives the use of the kept packet back to the game, which frees the block with the packet's empty vector; nothing
    /// when no packet is kept.
    pub fn drop_packet(&mut self) {
        if let Some(mut packet) = self.packet.take() {
            // SAFETY: the packet holds the one use the factory gave, which is given back once.
            unsafe { packet.release() };
        }
        self.forget_block();
    }

    /// Forgets the kept packet and its block without giving them back, so no game code runs for them; for a game that
    /// stopped.
    pub fn forget_packet(&mut self) {
        self.packet = None;
        self.forget_block();
    }

    fn forget_block(&mut self) {
        self.block = std::ptr::null_mut();
        self.capacity = 0;
    }
}

impl Default for ShapesHeld {
    fn default() -> Self {
        Self::new()
    }
}
