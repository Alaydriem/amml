use amml_abi::AmmlInputMode;

/// The device the game takes the pointer of a screen from.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputMode {
    Mouse,
    Touch,
    GamePad,
    MotionController,
    /// A value this crate does not know.
    Unknown,
}

impl From<AmmlInputMode> for InputMode {
    fn from(mode: AmmlInputMode) -> Self {
        match mode {
            AmmlInputMode::MOUSE => Self::Mouse,
            AmmlInputMode::TOUCH => Self::Touch,
            AmmlInputMode::GAME_PAD => Self::GamePad,
            AmmlInputMode::MOTION_CONTROLLER => Self::MotionController,
            _ => Self::Unknown,
        }
    }
}

impl From<InputMode> for AmmlInputMode {
    fn from(mode: InputMode) -> Self {
        match mode {
            InputMode::Mouse => Self::MOUSE,
            InputMode::Touch => Self::TOUCH,
            InputMode::GamePad => Self::GAME_PAD,
            InputMode::MotionController => Self::MOTION_CONTROLLER,
            InputMode::Unknown => Self::UNKNOWN,
        }
    }
}
