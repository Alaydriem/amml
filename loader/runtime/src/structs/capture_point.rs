/// Where in a frame a capture reads the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapturePoint {
    /// Before the first `FramePresent` listener: the game image alone (`GameOnly`).
    BeforeListeners,
    /// After the last listener of the normal band: the image with what the normal band drew (`WithOverlays`).
    AfterNormalBand,
}
