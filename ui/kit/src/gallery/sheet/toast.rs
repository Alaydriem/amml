use egui::{Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::widget::{Toast, ToastTone};

/// Paper: Mod manager, `Widget / Toast` (`DK5-0`). Five toasts in a 600 wide slot; the last one is
/// hovered, so it shows its close button.
pub struct ToastSheet;

impl ToastSheet {
    /// Sheet chrome: the slot the toasts stack in, and the gap between them.
    const SLOT_WIDTH: f32 = 600.0;
    const GAP: f32 = 12.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Toast", |ui| {
            ui.vertical(|ui| {
                ui.set_width(Self::SLOT_WIDTH);
                ui.spacing_mut().item_spacing = vec2(0.0, Self::GAP);
                let held = "Camera held by DejaVu.";
                let hint = "Change the rule in the panel under Conflicts";
                Toast::new(ToastTone::Danger, Sample::text(held))
                    .message(Sample::text("Freecam request denied."))
                    .hint(Sample::text(hint))
                    .age_ms(3000)
                    .remaining(0.35)
                    .show(ui);
                Toast::new(ToastTone::Danger, Sample::text(held))
                    .message(Sample::text("Freecam Toggle freecam blocked."))
                    .hint(Sample::text(hint))
                    .age_ms(0)
                    .remaining(0.85)
                    .show(ui);
                Toast::new(ToastTone::Warning, Sample::text("New key conflict."))
                    .message(Sample::text("Ctrl + B is bound by Freecam and Light Level."))
                    .age_ms(0)
                    .remaining(0.92)
                    .show(ui);
                Toast::new(ToastTone::Success, Sample::text("Recording saved."))
                    .message(Sample::text("Open it from Replays."))
                    .age_ms(12_000)
                    .remaining(0.1)
                    .show(ui);
                Toast::new(ToastTone::Info, Sample::text("Settings saved."))
                    .age_ms(5000)
                    .remaining(0.5)
                    .dismissable(true)
                    .show(ui);
            });
        });
    }
}
