use egui::{Color32, Margin};

use crate::draw::{Corner, Draw, TypeSize};
use crate::tokens::rule::Elevation;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Card frame + surfaces`, "SURFACE LEVELS".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceLevel {
    /// `--color-bg-app`: the window behind everything.
    App,
    /// `--color-bg-panel`: a docked panel.
    Panel,
    /// `--color-bg-raised`: a surface on a panel, and every floating surface.
    Raised,
    /// `--color-card`: a see-through card over the game picture.
    Card,
    /// `--color-pill`: a see-through pill over the game picture. Round, with no border.
    Pill,
}

impl SurfaceLevel {
    pub fn fill(self) -> Color32 {
        match self {
            Self::App => <color::BgApp as ColorToken>::VALUE,
            Self::Panel => <color::BgPanel as ColorToken>::VALUE,
            Self::Raised => <color::BgRaised as ColorToken>::VALUE,
            Self::Card => <color::Card as ColorToken>::VALUE,
            Self::Pill => <color::Pill as ColorToken>::VALUE,
        }
    }

    /// `--color-line-strong` when it floats, else `--color-line`. A card keeps `--color-line` over
    /// the game picture, and a pill has no border.
    pub fn border(self, elevation: Elevation) -> Option<Color32> {
        match (self, elevation) {
            (Self::Pill, _) => None,
            (Self::Card, _) | (_, Elevation::Flat) => Some(<color::Line as ColorToken>::VALUE),
            (_, Elevation::Floating) => Some(<color::LineStrong as ColorToken>::VALUE),
        }
    }

    pub fn corner(self) -> Corner {
        match self {
            Self::Pill => Corner::Full,
            _ => Corner::Panel,
        }
    }

    /// `--spacing-l` all round; a pill is `--spacing-m` across and `--spacing-row` high on caption text.
    pub fn margin(self) -> Margin {
        match self {
            Self::Pill => {
                let row = <spacing::Row as SpacingToken>::VALUE;
                Draw::margin(<spacing::M as SpacingToken>::VALUE, (row - TypeSize::Caption.line_height()) / 2.0)
            }
            _ => {
                let padding = <spacing::L as SpacingToken>::VALUE;
                Draw::margin(padding, padding)
            }
        }
    }
}
