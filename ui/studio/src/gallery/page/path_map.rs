use amml_ui::gallery::SheetGrid;
use egui::{Ui, vec2};

use super::ViewerSamples;
use crate::component::viewer::PathMap;
use crate::gallery::GalleryState;

/// Paper: Editor components, `Path map` (`46E-0`). A selected key, the camera track hidden, the Original replay, and
/// the scene camera driving.
pub struct PathMapPage;

impl PathMapPage {
    /// Paper: the sheet is 1240 wide, the columns 40 apart.
    pub const WIDTH: f32 = 1240.0;
    const GAP: f32 = 40.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Path map", |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(Self::GAP, 0.0);
                let columns = [
                    ("KEY SELECTED", ViewerSamples::path_map("Path · Arrival", true, Some(2), false, false)),
                    ("CAMERA TRACK HIDDEN", ViewerSamples::path_map("Path · Arrival", true, None, true, false)),
                    ("ORIGINAL TAB", ViewerSamples::path_map("Path · Original (no camera)", false, None, false, false)),
                    ("SCENE CAMERA DRIVES", ViewerSamples::path_map("Path · Arrival", true, None, false, true)),
                ];
                for (head, view) in columns {
                    SheetGrid::column(ui, head, PathMap::SIZE.x, |ui| {
                        PathMap::new(&view).show(ui);
                    });
                }
            });
        });
    }
}
