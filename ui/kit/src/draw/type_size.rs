use crate::tokens::{LineHeightToken, TextSizeToken, line_height, text_size};

/// A step of the Paper type scale: its `--text-*` size and its `--leading-*` line height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeSize {
    Micro,
    Caption,
    Note,
    Help,
    Label,
    Body,
    Title,
    Heading,
    Display,
}

impl TypeSize {
    pub fn points(self) -> f32 {
        match self {
            Self::Micro => <text_size::Micro as TextSizeToken>::VALUE,
            Self::Caption => <text_size::Caption as TextSizeToken>::VALUE,
            Self::Note => <text_size::Note as TextSizeToken>::VALUE,
            Self::Help => <text_size::Help as TextSizeToken>::VALUE,
            Self::Label => <text_size::Label as TextSizeToken>::VALUE,
            Self::Body => <text_size::Body as TextSizeToken>::VALUE,
            Self::Title => <text_size::Title as TextSizeToken>::VALUE,
            Self::Heading => <text_size::Heading as TextSizeToken>::VALUE,
            Self::Display => <text_size::Display as TextSizeToken>::VALUE,
        }
    }

    pub fn line_height(self) -> f32 {
        match self {
            Self::Micro => <line_height::Micro as LineHeightToken>::VALUE,
            Self::Caption => <line_height::Caption as LineHeightToken>::VALUE,
            Self::Note => <line_height::Note as LineHeightToken>::VALUE,
            Self::Help => <line_height::Help as LineHeightToken>::VALUE,
            Self::Label => <line_height::Label as LineHeightToken>::VALUE,
            Self::Body => <line_height::Body as LineHeightToken>::VALUE,
            Self::Title => <line_height::Title as LineHeightToken>::VALUE,
            Self::Heading => <line_height::Heading as LineHeightToken>::VALUE,
            Self::Display => <line_height::Display as LineHeightToken>::VALUE,
        }
    }
}
