use amml_runtime::ShotRequest;

/// What one back-buffer copy is for: a screenshot, or one frame of a capture stream (the present it reserved a ring
/// slot for).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptureJob {
    Shot(ShotRequest),
    Stream { stream: u64, present: u64 },
}
