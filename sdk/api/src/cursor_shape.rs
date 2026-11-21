use amml_abi::AmmlCursorShape;

/// A cursor shape a `WindowCursor` listener sets over the game window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorShape {
    Arrow,
    Hand,
    Text,
}

impl From<CursorShape> for AmmlCursorShape {
    fn from(shape: CursorShape) -> Self {
        match shape {
            CursorShape::Arrow => Self::ARROW,
            CursorShape::Hand => Self::HAND,
            CursorShape::Text => Self::TEXT,
        }
    }
}
