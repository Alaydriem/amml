use amml_abi::AmmlMouseButton;

/// One injected input that reaches the game at the next frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InjectedOp {
    Key {
        key: u16,
        down: bool,
    },
    Button {
        button: AmmlMouseButton,
        down: bool,
    },
    /// Whole notches only; the fraction stays with the lease.
    Wheel {
        notches: f32,
    },
}
