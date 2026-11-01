use amml_ui::tokens::{ColorToken, color};
use egui::Color32;

use crate::tokens::color as studio;

/// Paper: `3TE-0` section A. The colour of the mode chip's dot and border.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewerTone {
    /// Editing with the free camera: `--color-text-dim`.
    Edit,
    /// Editing while the replay plays: `--color-success`.
    Playing,
    /// Preview on but paused, or outside a scene: `--color-warning`.
    Paused,
    /// Preview playing, the scene camera drives: `--color-scene-camera`.
    SceneCamera,
    /// The camera follows a player: `--color-accent`.
    Following,
}

impl ViewerTone {
    // Paper: the mode chip's border is its dot colour at 70 %
    const BORDER_ALPHA: u8 = 0xB3;

    /// The mode chip's border colour.
    pub fn border(self) -> Color32 {
        self.color().gamma_multiply_u8(Self::BORDER_ALPHA)
    }

    pub fn color(self) -> Color32 {
        match self {
            Self::Edit => <color::TextDim as ColorToken>::VALUE,
            Self::Playing => <color::Success as ColorToken>::VALUE,
            Self::Paused => <color::Warning as ColorToken>::VALUE,
            Self::SceneCamera => <studio::SceneCamera as ColorToken>::VALUE,
            Self::Following => <color::Accent as ColorToken>::VALUE,
        }
    }
}
