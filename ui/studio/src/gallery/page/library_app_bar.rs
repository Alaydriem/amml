use amml_ui::gallery::{Sample, SheetGrid};
use egui::Ui;

use crate::component::library::{LibraryAppBar, LibraryAppBarTab, LibraryAppBarView};
use crate::gallery::GalleryState;

/// Paper: Editor components, `Editor / Library app bar` (`G4L-0`). The bar with the Library tab active and with the
/// Settings tab active.
pub struct LibraryAppBarPage;

impl LibraryAppBarPage {
    /// Sheet chrome: the width of one bar.
    const BAR: f32 = 1000.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Library app bar", |ui| {
            for (caption, tab) in
                [("LIBRARY ACTIVE", LibraryAppBarTab::Library), ("SETTINGS ACTIVE", LibraryAppBarTab::Settings)]
            {
                let view = LibraryAppBarView {
                    monogram: Sample::text("DV"),
                    name: Sample::text("DejaVu"),
                    library: Sample::text("Library"),
                    settings: Sample::text("Settings"),
                    refresh: Sample::text("Refresh"),
                    open_file: Sample::text("Open file…"),
                    back: Some(Sample::text("Back to replay")),
                    tab,
                };
                SheetGrid::section(ui, caption, |ui| {
                    ui.scope(|ui| {
                        ui.set_width(Self::BAR);
                        LibraryAppBar::new(&view).show(ui);
                    });
                });
            }
        });
    }
}
