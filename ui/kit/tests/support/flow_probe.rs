use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use egui::{Rect, Sense, Ui, vec2};
use egui_kittest::Harness;

// what a widget took and where the next widget in the same ui went
#[derive(Debug)]
pub struct FlowLog {
    pub widget: Rect,
    pub next: Rect,
}

// shows one widget, then a small block under it in the same ui
pub struct FlowProbe;

impl FlowProbe {
    pub fn run(mut show: impl FnMut(&mut Ui) + 'static) -> FlowLog {
        let mut harness = Harness::builder()
            .with_size(vec2(800.0, 600.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, log: &mut FlowLog| {
                    show(ui);
                    log.widget = ui.min_rect();
                    log.next = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover()).0;
                },
                FlowLog { widget: Rect::NOTHING, next: Rect::NOTHING },
            );
        harness.run();
        let FlowLog { widget, next } = harness.state();
        FlowLog { widget: *widget, next: *next }
    }

    // the next widget starts under the whole widget
    pub fn assert_below(log: &FlowLog) {
        assert!(log.next.top() >= log.widget.bottom(), "the next widget starts inside this one: {log:?}");
    }
}
