use amml_studio_ui::gallery::{GalleryPage, GalleryState};
use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use egui::vec2;
use egui_kittest::Harness;

// renders one gallery page and compares it with tests/snapshots/<page>.png
// (`mise run amml-studio-ui:snapshots` writes the baselines)
pub struct PageSnapshot;

impl PageSnapshot {
    // a few fixed steps, not `run`: a spinner asks for a repaint every frame
    const STEPS: usize = 4;

    pub fn check(page: GalleryPage) {
        let mut harness = Harness::builder()
            .with_size(vec2(page.width(), 400.0))
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::Embedded))
            .build_ui_state(move |ui, state: &mut GalleryState| page.show(ui, state), GalleryState::default());
        harness.run_steps(Self::STEPS);
        // twice: a modal's body scroll area takes its height from the screen it had on the pass before
        for _ in 0..2 {
            harness.fit_contents();
            harness.run_steps(Self::STEPS);
        }
        harness.snapshot(page.file_name());
    }
}
