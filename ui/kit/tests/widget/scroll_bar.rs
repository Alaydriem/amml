use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::ScrollBar;
use egui::epaint::RectShape;
use egui::{ScrollArea, Shape, vec2};
use egui_kittest::Harness;

use crate::support::PaintCapture;

// a 200 high scroll area of 40 labels, with or without the track; returns what was painted
struct ScrollProbe;

impl ScrollProbe {
    fn shapes(track: bool) -> Vec<Shape> {
        let mut harness: Harness<'static, ()> = Harness::builder()
            .with_size(vec2(300.0, 300.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                move |ui, _: &mut ()| {
                    ScrollBar::show(ui, ScrollArea::vertical().max_height(200.0), track, |ui| {
                        for index in 0..40 {
                            ui.label(format!("Row {index}"));
                        }
                    });
                },
                (),
            );
        harness.run();
        harness.output().shapes.iter().map(|clipped| clipped.shape.clone()).collect()
    }

    // the bars as wide as `--spacing-scrollbar`
    fn bars(shapes: &[Shape]) -> Vec<RectShape> {
        let width = <spacing::Scrollbar as SpacingToken>::VALUE;
        PaintCapture::rects(shapes).into_iter().filter(|r| (r.rect.width() - width).abs() < 0.01).collect()
    }
}

#[test]
fn style_is_thin_and_token_coloured() {
    let shapes = ScrollProbe::shapes(true);
    let bars = ScrollProbe::bars(&shapes);
    let thumb = bars.iter().find(|r| r.fill == color::LineStrong::VALUE);
    let Some(thumb) = thumb else { panic!("no `--spacing-scrollbar` wide thumb in `--color-line-strong`: {bars:?}") };
    let track = bars.iter().find(|r| r.fill == color::Line::VALUE);
    let Some(track) = track else { panic!("no `--color-line` track: {bars:?}") };
    assert!(track.rect.contains_rect(thumb.rect), "the thumb is not on the track");
    assert!(thumb.rect.height() < track.rect.height(), "the thumb is the whole track");

    // the lane area hides the track and keeps the thumb
    let bars = ScrollProbe::bars(&ScrollProbe::shapes(false));
    assert!(bars.iter().any(|r| r.fill == color::LineStrong::VALUE), "no thumb without the track");
    assert!(bars.iter().all(|r| r.fill != color::Line::VALUE), "the track shows when hidden");
}

#[test]
fn content_keeps_the_theme_visuals() {
    let mut harness: Harness<'static, Option<egui::Color32>> = Harness::builder()
        .with_size(vec2(300.0, 300.0))
        .allow_missing_glyphs()
        .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
        .build_ui_state(
            |ui, seen: &mut Option<egui::Color32>| {
                ScrollBar::show(ui, ScrollArea::vertical().max_height(200.0), true, |ui| {
                    *seen = Some(ui.visuals().widgets.inactive.bg_fill);
                });
            },
            None,
        );
    harness.run();
    assert_eq!(*harness.state(), Some(color::BgField::VALUE), "the bar colours reached the content");
}
