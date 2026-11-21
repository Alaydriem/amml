/// Where the overlay checks of the API test mod are. The square is drawn from the enable; the checks
/// start in the joined world and run one phase at a time on the client ticks, beside the input checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayPhase {
    /// Before the first `WorldJoined`.
    Idle,
    /// The world is joined; the checks wait for it to settle and for the first drawn frame.
    Settle,
    /// Both screenshots were asked; the checks wait for their `ScreenshotWritten`.
    Shots,
    /// A capture stream is open and its frames are taken.
    Capture,
    /// The stream stays open and no frame is taken, so the loader skips presents.
    CaptureIdle,
    /// A client size was requested; the checks wait for its `WindowSize` and then give the size back.
    RequestSize,
    /// The checks wait for the window events of the in-game task's resizes, click and character.
    Window,
    /// Frame times with the drawing listener.
    CostDraw,
    /// The drawing listener is removed; frame times with no `FramePresent` listener.
    CostBare,
    Done,
}

impl OverlayPhase {
    /// The phase after this one.
    pub fn next(self) -> Self {
        match self {
            Self::Idle => Self::Settle,
            Self::Settle => Self::Shots,
            Self::Shots => Self::Capture,
            Self::Capture => Self::CaptureIdle,
            Self::CaptureIdle => Self::RequestSize,
            Self::RequestSize => Self::Window,
            Self::Window => Self::CostDraw,
            Self::CostDraw => Self::CostBare,
            Self::CostBare | Self::Done => Self::Done,
        }
    }
}
