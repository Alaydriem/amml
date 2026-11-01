use amml_ui::tokens::{ColorToken, color};
use egui::Color32;

use crate::tokens::selector::TrackColor;

/// The colour of a key header's stripe and dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyHeaderTone {
    /// One key, or a track: the track's colour.
    Track(TrackColor),
    /// Several keys of different tracks: the accent.
    Several,
}

impl KeyHeaderTone {
    pub fn color(self) -> Color32 {
        match self {
            Self::Track(track) => track.color(),
            Self::Several => <color::Accent as ColorToken>::VALUE,
        }
    }
}
