use amml_ui::gallery::{Sample, SheetGrid};
use egui::Ui;

use crate::component::library::{SourceList, SourceListView, SourceRow, SourceRowKind};
use crate::gallery::GalleryState;

/// Paper: Screen / Library (`DX-0`), the Sources block under the replay list: the automatic source and a folder
/// the person added.
pub struct SourceListPage;

impl SourceListPage {
    /// Sheet chrome: the block width.
    const LIST: f32 = 519.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Source list", |ui| {
            let view = SourceListView {
                title: Sample::text("SOURCES"),
                sources: vec![
                    SourceRow {
                        name: Sample::text("BVC recordings"),
                        kind: SourceRowKind::Automatic { tag: Sample::text("auto") },
                    },
                    SourceRow { name: Sample::text("D:\\Captures\\Minecraft"), kind: SourceRowKind::Folder },
                ],
                add: Sample::text("Add folder"),
            };
            SheetGrid::section(ui, "ONE AUTOMATIC SOURCE, ONE FOLDER", |ui| {
                ui.scope(|ui| {
                    ui.set_width(Self::LIST);
                    SourceList::new(&view).show(ui);
                });
            });
        });
    }
}
