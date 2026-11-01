use amml_i18n::text::UserText;
use egui::{Painter, Rect};

use crate::draw::{Corner, Draw};
use crate::icon::{Icon, IconSize};
use crate::tokens::{SpacingToken, spacing};
use crate::widget::{Avatar, AvatarImage, AvatarSize, ChipTone};

/// The part at the left of a page header.
pub enum PageHeaderLead {
    /// An icon on a tile in a chip tone's fill and text colour.
    Tile(Icon, ChipTone),
    /// A large avatar with the name's first letter, or the picture when there is one.
    Avatar(UserText, Option<AvatarImage>),
}

impl PageHeaderLead {
    /// The side of the square. The panel form's tile is `--spacing-tile`; every other lead is
    /// `--spacing-tile-large`.
    pub fn side(&self, panel: bool) -> f32 {
        match (self, panel) {
            (Self::Tile(..), true) => <spacing::Tile as SpacingToken>::VALUE,
            (Self::Tile(..), false) => <spacing::TileLarge as SpacingToken>::VALUE,
            (Self::Avatar(..), _) => AvatarSize::Large.side(),
        }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, panel: bool) {
        match self {
            Self::Tile(icon, tone) => {
                let corner = if panel { Corner::Control } else { Corner::Panel };
                Draw::fill(painter, rect, corner, tone.fill());
                icon.paint(painter, rect, IconSize::Regular, tone.text());
            }
            Self::Avatar(name, image) => Avatar::new(name, AvatarSize::Large).image(image.clone()).paint(painter, rect),
        }
    }
}
