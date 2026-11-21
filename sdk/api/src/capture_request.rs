use crate::{ShotLayers, Size};

/// What `Overlay::open_capture_stream` asks for: which image, one frame every `every_nth_present`
/// presents, how many frames the loader may hold (`ring`, 1 to 8) and the largest frame size
/// (`None` for no limit; the loader scales down by the smallest whole factor that fits).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaptureRequest {
    pub layers: ShotLayers,
    pub every_nth_present: u32,
    pub ring: u8,
    pub max_size: Option<Size>,
}
