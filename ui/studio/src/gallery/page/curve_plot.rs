use amml_ui::gallery::SheetGrid;
use amml_ui::tokens::ColorToken;
use egui::{Color32, Pos2, Sense, Ui, pos2, vec2};

use crate::component::easing::{CurvePlot, CurvePlotView};
use crate::gallery::GalleryState;
use crate::tokens::color as studio;

/// Paper: Widgets, `Widget / Curve plot` (`6IV-0`), section "Key tab". The plot of a camera key (custom curve with
/// handles), a FOV key and a time of day key, each in its track colour. The card around the plot is the easing
/// popover's (Task 8).
pub struct CurvePlotPage;

impl CurvePlotPage {
    /// Paper: the plot is 288 × 160; the cards are 352 wide.
    const PLOT: [f32; 2] = [288.0, 160.0];
    const COLUMN: f32 = 352.0;
    const STEPS: usize = 32;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Curve plot", |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(SheetGrid::COLUMN_GAP, 0.0);
                let camera = <studio::TrackCamera as ColorToken>::VALUE;
                let fov = <studio::TrackFov as ColorToken>::VALUE;
                let time = <studio::TrackTime as ColorToken>::VALUE;
                // the camera key's custom curve: Paper's handles at (117, 152) and (171, 8) in a 272 × 144 box
                let handles = [pos2(109.0 / 272.0, 0.0), pos2(163.0 / 272.0, 1.0)];
                let in_out_cubic =
                    |t: f32| if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 };
                Self::column(ui, "CAMERA KEY", camera, Some(handles), &in_out_cubic);
                Self::column(ui, "FOV KEY", fov, None, &|t: f32| 1.0 - (1.0 - t) * (1.0 - t));
                Self::column(ui, "TIME OF DAY KEY", time, None, &|t: f32| t * t);
            });
        });
    }

    fn column(ui: &mut Ui, head: &str, color: Color32, handles: Option<[Pos2; 2]>, ease: &dyn Fn(f32) -> f32) {
        let samples: Vec<f32> = (0..=Self::STEPS).map(|step| ease(step as f32 / Self::STEPS as f32)).collect();
        SheetGrid::column(ui, head, Self::COLUMN, |ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(Self::PLOT[0], Self::PLOT[1]), Sense::hover());
            let view = CurvePlotView { samples: &samples, color, handles };
            CurvePlot::new(&view).paint(ui.painter(), rect);
        });
    }
}
