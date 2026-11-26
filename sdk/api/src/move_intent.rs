use amml_abi::AmmlMoveIntent;

/// The movement a lease asks for: `forward` and `left` in -1.0 to 1.0, and the held actions.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct MoveIntent {
    pub forward: f32,
    pub left: f32,
    pub jump: bool,
    pub sneak: bool,
    pub sprint: bool,
}

impl From<AmmlMoveIntent> for MoveIntent {
    fn from(intent: AmmlMoveIntent) -> Self {
        Self {
            forward: intent.forward,
            left: intent.left,
            jump: intent.jump != 0,
            sneak: intent.sneak != 0,
            sprint: intent.sprint != 0,
        }
    }
}

impl From<MoveIntent> for AmmlMoveIntent {
    fn from(intent: MoveIntent) -> Self {
        Self {
            forward: intent.forward,
            left: intent.left,
            jump: u8::from(intent.jump),
            sneak: u8::from(intent.sneak),
            sprint: u8::from(intent.sprint),
            _reserved: 0,
        }
    }
}
