/// Where the in-world checks of the API test mod are; each phase runs on the client ticks of the
/// joined world until it is done or its time runs out, then hands over to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckPhase {
    /// The title screen; the checks wait for `WorldJoined`.
    Title,
    /// The world is joined; the checks wait for it to settle.
    Settle,
    Reads,
    HudHide,
    HudShow,
    MouseRelease,
    MouseCapture,
    KeyHold,
    Move,
    Turn,
    Wheel,
    Click,
    /// The X1 and X2 buttons, injected as the left click is.
    ClickX1,
    ClickX2,
    Gamepad,
    Pause,
    Done,
}

impl CheckPhase {
    /// The phase after this one.
    pub fn next(self) -> Self {
        match self {
            Self::Title => Self::Settle,
            Self::Settle => Self::Reads,
            Self::Reads => Self::HudHide,
            Self::HudHide => Self::HudShow,
            Self::HudShow => Self::MouseRelease,
            Self::MouseRelease => Self::MouseCapture,
            Self::MouseCapture => Self::KeyHold,
            Self::KeyHold => Self::Move,
            Self::Move => Self::Turn,
            Self::Turn => Self::Wheel,
            Self::Wheel => Self::Click,
            Self::Click => Self::ClickX1,
            Self::ClickX1 => Self::ClickX2,
            Self::ClickX2 => Self::Gamepad,
            Self::Gamepad => Self::Pause,
            Self::Pause | Self::Done => Self::Done,
        }
    }
}
