use amml_ui::gallery::{Sample, SheetGrid};
use egui::Ui;

use crate::component::viewer::EditLockBanner;
use crate::gallery::GalleryState;

/// Paper: Editor components, `Editor / Edit lock banner` (`G7V-0`). The pill while the scene camera drives.
pub struct EditLockBannerPage;

impl EditLockBannerPage {
    /// Sheet chrome: the pill width.
    const PILL: f32 = 402.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Edit lock banner", |ui| {
            SheetGrid::section(ui, "SCENE CAMERA", |ui| {
                EditLockBanner::new(Sample::text("Locked: Scene camera drives · pause to edit")).show(ui, Self::PILL);
            });
        });
    }
}
