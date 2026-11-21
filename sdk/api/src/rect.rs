use amml_abi::AmmlRect;

/// A rectangle in GUI units, the units of the UI layout of the game: `x` and `y` are the top left corner.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<AmmlRect> for Rect {
    fn from(rect: AmmlRect) -> Self {
        Self { x: rect.x, y: rect.y, width: rect.width, height: rect.height }
    }
}

impl From<Rect> for AmmlRect {
    fn from(rect: Rect) -> Self {
        Self { x: rect.x, y: rect.y, width: rect.width, height: rect.height }
    }
}
