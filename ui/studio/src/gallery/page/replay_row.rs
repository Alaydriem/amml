use amml_ui::gallery::{Sample, SheetGrid};
use egui::Ui;

use crate::component::library::{ReplayRow, ReplayRowNote, ReplayRowView};
use crate::gallery::GalleryState;

/// Paper: Screen / Library (`DX-0`), the rows of the replay list; `65T-0` for the broken row. A row at rest, under
/// the pointer, selected and broken. The studio bundles no pictures, so the thumbnails are plain cards.
pub struct ReplayRowPage;

impl ReplayRowPage {
    /// Sheet chrome: the row width.
    const ROW: f32 = 539.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Replay row", |ui| {
            let rows = [
                ("REST", false, false, false, "Castle build, day 4", "00:48:12"),
                ("HOVER", true, false, false, "Ender dragon attempt", "01:02:51"),
                ("SELECTED", false, true, false, "Nether trip", "02:14:08"),
                ("BROKEN", false, false, true, "Old realm backup", "00:00:00"),
            ];
            for (caption, hovered, selected, broken, name, length) in rows {
                let facts = if broken {
                    ["Sep 30", "0 players", "0 scenes", "0 bookmarks"]
                } else {
                    ["Today, 14:02", "6 players", "5 scenes", "3 bookmarks"]
                };
                let note = if broken {
                    ReplayRowNote::Broken(Sample::text("missing capture"))
                } else {
                    ReplayRowNote::Source(vec![Sample::text("BVC"), Sample::text("recent")])
                };
                let view = ReplayRowView {
                    name: Sample::text(name),
                    length: Sample::text(length),
                    facts: facts.iter().map(|fact| Sample::text(fact)).collect(),
                    note,
                    thumbnail: None,
                    selected,
                };
                SheetGrid::section(ui, caption, |ui| {
                    ui.scope(|ui| {
                        ui.set_width(Self::ROW);
                        ReplayRow::new(&view).hovered(hovered).show(ui);
                    });
                });
            }
        });
    }
}
