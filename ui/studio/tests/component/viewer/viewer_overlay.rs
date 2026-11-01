use amml_studio_ui::component::viewer::ViewerOverlay;
use amml_ui::tokens::{ColorToken, color};
use egui::{Rect, Shape, pos2, vec2};

use crate::glyph::words;
use crate::support::PaintCapture;

fn viewer() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 450.0))
}

// the width of the warning-coloured progress fill
fn bar_width(progress: f32) -> f32 {
    let overlay = ViewerOverlay::Rebuilding { progress, detail: words("from checkpoint 00:00:10") };
    let shapes = PaintCapture::shapes(|painter| overlay.paint(painter, viewer()));
    let warning = <color::Warning as ColorToken>::VALUE;
    shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Rect(rect) if rect.fill == warning => Some(rect.rect.width()),
            _ => None,
        })
        .sum()
}

#[test]
fn rebuild_overlay_clamps_progress() {
    let full = bar_width(1.0);
    assert!(full > 0.0);
    assert_eq!(bar_width(-1.0), 0.0);
    assert_eq!(bar_width(f32::NAN), 0.0);
    assert_eq!(bar_width(2.0), full);
    assert!((bar_width(0.5) - full / 2.0).abs() < 0.01);
}

#[test]
fn overlays_stay_inside_a_small_viewer() {
    let small = Rect::from_min_size(pos2(0.0, 0.0), vec2(320.0, 180.0));
    let overlays = [
        ViewerOverlay::Rebuilding { progress: 0.7, detail: words("from checkpoint 00:00:10") },
        ViewerOverlay::Joining {
            title: words("Joining “Nether trip”"),
            body: words("Attempt 1 of 5 · loopback server ready · 1.4 s"),
            hint: words("After 5 failed tries the mod asks you to join by hand."),
        },
        ViewerOverlay::Typing,
    ];
    for overlay in overlays {
        let shapes = PaintCapture::shapes(|painter| overlay.paint(painter, small));
        let bounds = PaintCapture::bounds(&shapes);
        assert!(small.contains_rect(bounds), "{overlay:?} paints {bounds:?}");
    }
}
