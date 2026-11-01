use egui::{Align2, Ui};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{PillTone, StatusPill};

/// Paper: Widgets, `Widget / Status pill` (`4JR-0`). The mode preview colour and the coverage
/// colours are amml-studio-ui tokens, so this page shows the preview pill in `--color-warning` and leaves
/// the coverage row to the amml-studio-ui gallery.
pub struct StatusPillSheet;

impl StatusPillSheet {
    const COLUMN: f32 = 140.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let height = <spacing::ControlSmall as SpacingToken>::VALUE;
        let preview = PillTone::Role(<color::Warning as ColorToken>::VALUE);
        let rows: [(&str, &[(PillTone, &str)]); 3] = [
            (
                "Render job",
                &[
                    (PillTone::Dim, "Queued"),
                    (PillTone::Success, "Done"),
                    (PillTone::Danger, "Failed"),
                    (PillTone::Faint, "Cancelled"),
                ],
            ),
            (
                "Scene status",
                &[(PillTone::Faint, "Draft"), (PillTone::Accent, "Ready"), (PillTone::Success, "Rendered")],
            ),
            (
                "Mode",
                &[
                    (PillTone::Success, "Edit"),
                    (preview, "Preview"),
                    (PillTone::Faint, "Read-only"),
                    (PillTone::Accent, "Busy"),
                ],
            ),
        ];
        SheetGrid::page(ui, "Status pill", |ui| {
            SheetGrid::panel(ui, |ui| {
                for (label, pills) in rows {
                    SheetGrid::row(ui, label, Self::COLUMN, height, pills.len(), |ui, index, cell| {
                        let (tone, words) = pills[index];
                        StatusPill::new(tone, Sample::text(words)).paint(
                            ui.painter(),
                            cell.left_center(),
                            Align2::LEFT_CENTER,
                        );
                    });
                }
            });
        });
    }
}
