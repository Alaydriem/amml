use crate::MouseButton;

/// What a `MouseInput` reports.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseAction {
    /// The pointer moved; read `x`, `y`, `dx` and `dy` of the event.
    Move,
    Button {
        button: MouseButton,
        down: bool,
    },
    /// One notch is 1.0; positive is away from the user.
    Wheel {
        notches: f32,
    },
    /// An action or a button this crate does not know.
    Other,
}
