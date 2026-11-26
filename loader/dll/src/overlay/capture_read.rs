use std::sync::Arc;

use super::{CaptureJob, CapturedImage};

/// One staging texture read by `FrameCapture::poll`: the jobs of its copy, and the image, or `None` when the copy
/// could not be read (not ready within `FrameCapture::MAP_WAIT_FRAMES` presents, or the map failed).
#[derive(Debug)]
pub struct CaptureRead {
    pub jobs: Vec<CaptureJob>,
    pub image: Option<Arc<CapturedImage>>,
}
