use std::ffi::c_void;

use amml_msvc::SharedPtr;
use amml_runtime::FreeCameraBook;

/// What the free camera holds under its lock: which mod holds the free camera, and the camera instruction packet the
/// loader made in this world (one use of the game's control block), reused by every delivery until the world is left
/// or the holder is disabled.
pub struct FreeCameraHeld {
    pub book: FreeCameraBook,
    pub packet: Option<SharedPtr<c_void>>,
}

// SAFETY: the packet is used only under the adapter's lock, on the thread that holds it; the game's control block
// counts its uses with atomics, so the last use may be given back on any thread.
unsafe impl Send for FreeCameraHeld {}

impl FreeCameraHeld {
    pub fn new() -> Self {
        Self { book: FreeCameraBook::new(), packet: None }
    }

    /// Gives the use of the kept packet back to the game; nothing when no packet is kept.
    pub fn drop_packet(&mut self) {
        if let Some(mut packet) = self.packet.take() {
            // SAFETY: the packet holds the one use the factory gave, which is given back once.
            unsafe { packet.release() };
        }
    }

    /// Forgets the kept packet without giving its use back, so no game code runs for it; for a game that stopped.
    pub fn forget_packet(&mut self) {
        self.packet = None;
    }
}

impl Default for FreeCameraHeld {
    fn default() -> Self {
        Self::new()
    }
}
