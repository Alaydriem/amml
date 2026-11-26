use std::sync::Arc;

use amml_runtime::ShotRequest;

use super::{CaptureStream, CapturedImage};

/// Work for the loader's capture worker: a screenshot to encode and write, or a stream frame to convert into its
/// ring slot.
pub enum WriterJob {
    Shot { request: ShotRequest, image: Arc<CapturedImage> },
    Frame { stream: Arc<CaptureStream>, present: u64, image: Arc<CapturedImage> },
}
