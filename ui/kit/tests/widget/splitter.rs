use amml_ui::font::FontSource;
use amml_ui::state::InteractionState;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::Splitter;
use egui::{Id, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;

use crate::support::PaintCapture;

// what the splitter moves and where it was
#[derive(Debug)]
struct SplitLog {
    position: f32,
    rect: Rect,
}

// one splitter, moving `position` in 50..=150, or in `start..=end` for `harness_in`
struct SplitProbe;

impl SplitProbe {
    fn harness() -> Harness<'static, SplitLog> {
        Self::harness_in(50.0, 150.0)
    }

    fn harness_in(start: f32, end: f32) -> Harness<'static, SplitLog> {
        let mut harness = Harness::builder()
            .with_size(vec2(400.0, 300.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, log: &mut SplitLog| {
                    log.rect = Splitter::new(Id::unique(1), &mut log.position, start..=end).show(ui).rect;
                },
                SplitLog { position: 100.0, rect: Rect::NOTHING },
            );
        harness.run();
        harness
    }

    // presses at `from`, moves by `by` in small steps, and lets go
    fn drag(harness: &mut Harness<'static, SplitLog>, from: Pos2, by: f32) {
        harness.hover_at(from);
        harness.run();
        harness.drag_at(from);
        harness.step();
        let steps = 10;
        for step in 1..=steps {
            harness.hover_at(from + vec2(0.0, by * step as f32 / steps as f32));
            harness.step();
        }
        harness.drop_at(from + vec2(0.0, by));
        harness.run();
    }
}

#[test]
fn drag_moves_within_range() {
    let mut harness = SplitProbe::harness();
    let rect = harness.state().rect;
    assert_eq!(rect.height(), <spacing::Splitter as SpacingToken>::VALUE);
    SplitProbe::drag(&mut harness, rect.center(), 30.0);
    assert!((harness.state().position - 130.0).abs() <= 1.0, "{:?}", harness.state());
    let rect = harness.state().rect;
    SplitProbe::drag(&mut harness, rect.center(), 100.0);
    assert_eq!(harness.state().position, 150.0, "the drag is not held at the top of the range");
    let rect = harness.state().rect;
    SplitProbe::drag(&mut harness, rect.center(), -200.0);
    assert_eq!(harness.state().position, 50.0, "the drag is not held at the bottom of the range");
}

#[test]
fn reversed_range_holds_at_its_start() {
    // a window shorter than both minimum panes gives a range whose end is below its start
    let mut harness = SplitProbe::harness_in(200.0, 150.0);
    let rect = harness.state().rect;
    SplitProbe::drag(&mut harness, rect.center(), 20.0);
    assert_eq!(harness.state().position, 200.0);
}

#[test]
fn hover_and_drag_fills() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(320.0, <spacing::Splitter as SpacingToken>::VALUE));
    let looks = [
        (InteractionState::Rest, color::BgPanel::VALUE, color::LineStrong::VALUE),
        (InteractionState::Hover, color::BgHover::VALUE, color::TextFaint::VALUE),
        (InteractionState::Pressed, color::AccentSoft::VALUE, color::Accent::VALUE),
    ];
    for (state, bar, grip) in looks {
        let rects = PaintCapture::rects(&PaintCapture::shapes(|painter| Splitter::paint(painter, rect, state)));
        assert!(rects.iter().any(|r| r.rect == rect && r.fill == bar), "{state:?}: bar fill");
        let Some(grip_rect) = rects.iter().find(|r| r.fill == grip) else { panic!("{state:?}: no grip") };
        assert_eq!(grip_rect.rect.center(), rect.center(), "{state:?}: the grip is not centred");
        assert!(grip_rect.rect.width() > grip_rect.rect.height(), "{state:?}: the grip is not a flat bar");
    }
}
