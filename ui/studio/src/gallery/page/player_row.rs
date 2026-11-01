use amml_ui::gallery::{Sample, SheetGrid};
use egui::Ui;

use crate::component::inspector::{PlayerRow, PlayerRowView};
use crate::gallery::GalleryState;

/// Paper: Editor components, `Editor / Player row` (`G7X-0`). A row at rest, a row under the pointer and the row of
/// the followed player.
pub struct PlayerRowPage;

impl PlayerRowPage {
    /// Sheet chrome: the row width.
    const ROW: f32 = 414.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Player row", |ui| {
            let rows = [
                ("REST", "K", "Kestrel", "32 m from the camera", false, false),
                ("HOVER", "A", "AuroraMiner (recorder)", "9 m from the camera", false, true),
                ("FOLLOWING", "B", "basalt_bea", "7 m from the camera", true, false),
            ];
            for (caption, initial, name, distance, following, hover) in rows {
                let view = PlayerRowView {
                    initial: Sample::text(initial),
                    name: Sample::text(name),
                    distance: Sample::text(distance),
                    follow: Sample::text(if following { "Following" } else { "Follow" }),
                    go_to: Sample::text("Go to"),
                    following,
                };
                SheetGrid::section(ui, caption, |ui| {
                    ui.scope(|ui| {
                        ui.set_width(Self::ROW);
                        PlayerRow::new(&view).hovered(hover).show(ui);
                    });
                });
            }
        });
    }
}
