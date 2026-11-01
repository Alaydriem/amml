use amml_ui::font::FontSource;
use amml_ui::gallery::{GallerySheet, GalleryState};
use amml_ui::theme::Theme;
use egui::vec2;
use egui_kittest::Harness;

// renders one gallery sheet and compares it with tests/snapshots/<sheet>.png
// (`mise run amml-ui:snapshots` writes the baselines)
pub struct SheetSnapshot;

impl SheetSnapshot {
    // a few fixed steps, not `run`: a spinner asks for a repaint every frame
    const STEPS: usize = 4;

    pub fn check(sheet: GallerySheet) {
        let mut harness = Harness::builder()
            .with_size(vec2(GallerySheet::WIDTH, 400.0))
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::Embedded))
            .build_ui_state(move |ui, state: &mut GalleryState| sheet.show(ui, state), GalleryState::default());
        harness.run_steps(Self::STEPS);
        harness.fit_contents();
        harness.run_steps(Self::STEPS);
        harness.snapshot(sheet.file_name());
    }
}
