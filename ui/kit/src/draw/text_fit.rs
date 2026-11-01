use egui::text::TextWrapping;

/// How text fits its width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextFit {
    /// One line, as wide as the text.
    Line,
    /// Wraps into as many rows as it needs within the width.
    Wrap(f32),
    /// One row within the width, cut with an ellipsis.
    Truncate(f32),
}

impl TextFit {
    pub fn wrapping(self) -> TextWrapping {
        match self {
            Self::Line => TextWrapping::no_max_width(),
            Self::Wrap(width) => TextWrapping::wrap_at_width(width),
            Self::Truncate(width) => TextWrapping::truncate_at_width(width),
        }
    }
}
