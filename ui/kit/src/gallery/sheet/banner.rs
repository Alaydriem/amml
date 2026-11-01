use egui::Ui;

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::widget::{Banner, BannerTone, ButtonKind, KeyChord, TextButton};

/// Paper: Mod manager, `Widget / Banner` (`DK1-0`). The four tones in one panel.
pub struct BannerSheet;

impl BannerSheet {
    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Banner", |ui| {
            SheetGrid::panel(ui, |ui| {
                Banner::new(BannerTone::Info, Sample::text("No settings"))
                    .body(Sample::text("This mod has only keybinds. Change them on the Keybinds tab."))
                    .show(ui);
                let chord = KeyChord::new(vec![Sample::text("Ctrl"), Sample::text("B")]);
                Banner::new(BannerTone::Warning, Sample::text("1 key conflict."))
                    .chord(chord, Sample::text("is also bound by Light Level."))
                    .show(ui);
                Banner::new(BannerTone::Danger, Sample::text("Light Level did not load"))
                    .body(Sample::text("It needs LeviLamina 1.5.0 or later."))
                    .action(TextButton::new(ButtonKind::Secondary, Sample::text("Show log")))
                    .show(ui);
                Banner::new(BannerTone::Success, Sample::text("All keybinds are unique.")).show(ui);
            });
        });
    }
}
