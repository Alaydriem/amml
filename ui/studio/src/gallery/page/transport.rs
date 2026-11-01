use amml_i18n::text::{Text, UserText};
use egui::Ui;

use super::BarRows;
use crate::component::bar::{Transport, TransportLayout, TransportView};
use crate::gallery::GalleryState;

/// Paper: Editor components, `Transport` (`UW-0`). Paused and playing at 1490 (beside the inspector), wide at 1920
/// with the replay length, and compact at 1236 (a 1280 window beside the inspector rail).
pub struct TransportPage;

impl TransportPage {
    /// Paper: the widest transport row.
    pub const WIDTH: f32 = 1920.0;
    // Paper: the row widths
    const BESIDE_INSPECTOR: f32 = 1490.0;
    const BESIDE_RAIL: f32 = 1236.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            let rows = [
                ("PAUSED", false, false, Self::BESIDE_INSPECTOR, TransportLayout::Full),
                ("PLAYING IN PREVIEW", true, false, Self::BESIDE_INSPECTOR, TransportLayout::Full),
                ("WIDE", false, true, Self::WIDTH, TransportLayout::Full),
                ("COMPACT", false, false, Self::BESIDE_RAIL, TransportLayout::Compact),
            ];
            for (caption, playing, wide, width, layout) in rows {
                let view = Self::view(playing, wide);
                BarRows::row(ui, caption, width, |ui| {
                    Transport::new(&view).layout(layout).show(ui);
                });
            }
        });
    }

    fn words(text: &str) -> Text {
        Text::user(UserText::from_model(text.to_owned()))
    }

    fn view(playing: bool, wide: bool) -> TransportView {
        TransportView {
            playing,
            timecode: Self::words(if playing { "00:00:31.12" } else { "00:00:32.95" }),
            duration: wide.then(|| Self::words("02:14:08.40")),
            speed: Self::words(if playing { "2x" } else { "1x" }),
            looping: playing,
            snap: true,
            guides: playing,
            zoom: 0.65,
        }
    }
}
