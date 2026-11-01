use crate::tokens::{TextSizeToken, text_size};

/// The two icon sizes in Paper: `--text-icon` and `--text-icon-small`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconSize {
    Regular,
    Small,
}

impl IconSize {
    pub fn points(self) -> f32 {
        match self {
            Self::Regular => <text_size::Icon as TextSizeToken>::VALUE,
            Self::Small => <text_size::IconSmall as TextSizeToken>::VALUE,
        }
    }
}
