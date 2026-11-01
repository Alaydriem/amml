use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use egui::{Key, Rect, Response, Ui};
use egui_kittest::Harness;

// what a widget's responses said over a harness run, and the value it edits
#[derive(Debug)]
pub struct ClickLog<T> {
    pub clicks: usize,
    pub changes: usize,
    pub rect: Rect,
    pub value: T,
}

// shows one widget per pass and counts its clicks and changes
pub struct ClickProbe;

impl ClickProbe {
    pub fn harness(mut show: impl FnMut(&mut Ui) -> Response + 'static) -> Harness<'static, ClickLog<()>> {
        Self::harness_with((), move |ui, _| show(ui))
    }

    // the widget edits `value`, which the log keeps between passes
    pub fn harness_with<T: 'static>(
        value: T,
        mut show: impl FnMut(&mut Ui, &mut T) -> Response + 'static,
    ) -> Harness<'static, ClickLog<T>> {
        // built-in fonts have no icon glyphs; the snapshot tests check glyphs with the real fonts
        let mut harness = Harness::builder()
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, log: &mut ClickLog<T>| {
                    let response = show(ui, &mut log.value);
                    log.clicks += usize::from(response.clicked());
                    log.changes += usize::from(response.changed());
                    log.rect = response.rect;
                },
                ClickLog { clicks: 0, changes: 0, rect: Rect::NOTHING, value },
            );
        harness.run();
        harness
    }

    pub fn click<T>(harness: &mut Harness<'static, ClickLog<T>>) {
        let centre = harness.state().rect.center();
        Self::click_at(harness, centre);
    }

    // one frame between press and release: `run` while held can pass egui's longest click when
    // something (a tooltip) keeps asking for frames
    pub fn click_at<T>(harness: &mut Harness<'static, ClickLog<T>>, at: egui::Pos2) {
        harness.hover_at(at);
        harness.run();
        harness.drag_at(at);
        harness.step();
        harness.drop_at(at);
        harness.run();
    }

    // a right click at `at`, one frame between press and release
    pub fn secondary_click_at<T>(harness: &mut Harness<'static, ClickLog<T>>, at: egui::Pos2) {
        harness.hover_at(at);
        harness.run();
        for pressed in [true, false] {
            harness.event(egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Secondary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            harness.step();
        }
        harness.run();
    }

    // every text the last frame painted
    pub fn texts<T>(harness: &Harness<'static, ClickLog<T>>) -> Vec<String> {
        let shapes: Vec<egui::Shape> = harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect();
        crate::support::ShapeText::all(&shapes)
    }

    // the rect of the first painted text equal to `text`
    pub fn text_rect<T>(harness: &Harness<'static, ClickLog<T>>, text: &str) -> Option<Rect> {
        harness.output().shapes.iter().find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.text() == text => {
                Some(shape.galley.rect.translate(shape.pos.to_vec2()))
            }
            _ => None,
        })
    }

    // Tab to the first focusable widget, then press `key`
    pub fn tab_and_press<T>(harness: &mut Harness<'static, ClickLog<T>>, key: Key) {
        harness.key_press(Key::Tab);
        harness.run();
        harness.key_press(key);
        harness.run();
    }
}
