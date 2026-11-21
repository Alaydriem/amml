use amml_abi::AmmlTextAlign;

/// Where a text sits along the width of the rectangle of a `UiDraw::text` call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl From<TextAlign> for AmmlTextAlign {
    fn from(align: TextAlign) -> Self {
        match align {
            TextAlign::Left => Self::LEFT,
            TextAlign::Center => Self::CENTER,
            TextAlign::Right => Self::RIGHT,
        }
    }
}
