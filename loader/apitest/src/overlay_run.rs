use std::path::PathBuf;
use std::time::Instant;

use amml_api::{CaptureStream, Listener, Size};

use crate::OverlayPhase;

/// The state of one run of the overlay checks: the phase, its tick count, the drawing listener, the
/// screenshot files and the capture stream. Dropping it removes the listener and closes the stream.
pub struct OverlayRun {
    pub phase: OverlayPhase,
    /// Client ticks since the phase began.
    pub ticks: u32,
    /// Whether the phase sent its request, and when.
    pub sent: bool,
    pub sent_at: Option<Instant>,
    /// The `FramePresent` listener that draws the square, from the enable until the `CostBare` phase.
    pub draw: Option<Listener>,
    /// The `GameOnly` and `WithOverlays` screenshot files.
    pub shots: Vec<PathBuf>,
    pub stream: Option<CaptureStream>,
    /// Capture frames taken, presents the stream skipped, and the `present_index` of the last frame.
    pub frames: u32,
    pub skipped: u64,
    pub last_index: Option<u64>,
    /// The size of the capture frames and whether the square shows in them.
    pub frame_size: Option<Size>,
    pub frame_magenta: Option<bool>,
}

impl Default for OverlayRun {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayRun {
    pub const fn new() -> Self {
        Self {
            phase: OverlayPhase::Idle,
            ticks: 0,
            sent: false,
            sent_at: None,
            draw: None,
            shots: Vec::new(),
            stream: None,
            frames: 0,
            skipped: 0,
            last_index: None,
            frame_size: None,
            frame_magenta: None,
        }
    }

    /// Moves to the next phase.
    pub fn advance(&mut self) {
        self.phase = self.phase.next();
        self.ticks = 0;
        self.sent = false;
        self.sent_at = None;
    }
}
