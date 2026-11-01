use amml_ui::gallery::{Sample, SheetGrid};
use egui::Ui;

use crate::component::inspector::{KeyHeader, KeyHeaderTone, KeyHeaderView};
use crate::gallery::GalleryState;
use crate::tokens::selector::TrackColor;

/// Paper: Editor components, `Editor / Key header` (`GAI-0`). One header per track, one for several keys and one
/// for a track.
pub struct KeyHeaderPage;

impl KeyHeaderPage {
    /// Sheet chrome: the header width.
    const HEADER: f32 = 398.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Key header", |ui| {
            let headers = [
                ("CAMERA", KeyHeaderTone::Track(TrackColor::Camera), "Camera key", "00:00:34.70 · tick 694", "#4 of 6"),
                ("FOV", KeyHeaderTone::Track(TrackColor::Fov), "FOV key", "00:00:31.60 · tick 632", "#2 of 4"),
                (
                    "TIME OF DAY",
                    KeyHeaderTone::Track(TrackColor::Time),
                    "Time of day key",
                    "00:00:28.80 · tick 576",
                    "#1 of 2",
                ),
                (
                    "WEATHER",
                    KeyHeaderTone::Track(TrackColor::Weather),
                    "Weather key",
                    "00:00:33.40 · tick 668",
                    "#2 of 2",
                ),
                (
                    "TIMELAPSE",
                    KeyHeaderTone::Track(TrackColor::Timelapse),
                    "Timelapse key",
                    "00:00:33.00 · tick 660",
                    "#2 of 3",
                ),
                ("SEVERAL KEYS", KeyHeaderTone::Several, "3 keys selected", "Camera, FOV", "multi"),
                ("TRACK", KeyHeaderTone::Track(TrackColor::Camera), "Camera track", "Arrival", "6 keys"),
            ];
            for (caption, tone, title, detail, chip) in headers {
                let view = KeyHeaderView {
                    title: Sample::text(title),
                    detail: Sample::text(detail),
                    chip: Sample::text(chip),
                    tone,
                };
                SheetGrid::section(ui, caption, |ui| {
                    ui.scope(|ui| {
                        ui.set_width(Self::HEADER);
                        KeyHeader::new(&view).show(ui);
                    });
                });
            }
        });
    }
}
