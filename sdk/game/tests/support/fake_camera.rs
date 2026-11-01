use amml_game::{EntryId, GameEntries};

use super::FakeEntries;

/// A fake `Camera`: 16 padding bytes, then the forward vector.
#[allow(dead_code)]
#[repr(C)]
pub struct FakeCamera {
    pad: [f32; 4],
    forward: [f32; 3],
}

impl FakeCamera {
    pub fn new(forward: [f32; 3]) -> Self {
        Self { pad: [0.0; 4], forward }
    }

    /// Entries with the offsets of the camera and view object layouts.
    pub fn entries() -> GameEntries {
        FakeEntries::build(&[
            (EntryId::CAMERA_FORWARD, 16),
            (EntryId::VIEW_RENDER_OBJECT_VIEW_DATA, 32),
            (EntryId::VIEW_RENDER_DATA_CAMERA_POS, 8),
        ])
    }
}
