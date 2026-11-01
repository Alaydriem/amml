use std::sync::LazyLock;

use egui::{FontFamily, FontId};

use crate::tokens::{FontFamilyToken, FontWeightToken, font_family, font_weight};

static SEMIBOLD: LazyLock<FontFamily> = LazyLock::new(|| {
    let name = format!(
        "{} {}",
        <font_family::Ui as FontFamilyToken>::VALUE,
        <font_weight::Semibold as FontWeightToken>::VALUE
    );
    FontFamily::Name(name.into())
});
static ICONS: LazyLock<FontFamily> =
    LazyLock::new(|| FontFamily::Name(<font_family::Icons as FontFamilyToken>::VALUE.into()));

/// The three faces Paper draws with: Nordwand Mono at the regular and semibold weights, and Segoe Fluent Icons.
/// The named families are built once, so a font id costs a reference count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFace {
    Regular,
    Semibold,
    Icons,
}

impl FontFace {
    pub fn family(self) -> FontFamily {
        match self {
            Self::Regular => FontFamily::Proportional,
            Self::Semibold => SEMIBOLD.clone(),
            Self::Icons => ICONS.clone(),
        }
    }

    pub fn id(self, size: f32) -> FontId {
        FontId::new(size, self.family())
    }
}
