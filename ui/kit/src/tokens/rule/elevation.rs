use egui::epaint::Shadow;

use crate::tokens::{ColorToken, color};

/// How far a surface floats above the one below it. Paper has no shadow token type, so the
/// floating shadow, `0 8px 24px --color-shadow`, is written here once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Elevation {
    /// On the surface below: no shadow.
    Flat,
    /// Menus, popovers and dialogs.
    Floating,
}

impl Elevation {
    pub fn shadow(self) -> Shadow {
        match self {
            Self::Flat => Shadow::NONE,
            Self::Floating => {
                Shadow { offset: [0, 8], blur: 24, spread: 0, color: <color::Shadow as ColorToken>::VALUE }
            }
        }
    }
}
