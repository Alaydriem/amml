use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{Surface, SurfaceLevel};
use egui::{Rect, pos2, vec2};

use crate::support::PaintCapture;

struct SurfacePaint;

impl SurfacePaint {
    const RECT: Rect = Rect::from_min_max(pos2(20.0, 20.0), pos2(220.0, 116.0));

    fn rects(level: SurfaceLevel, elevation: Elevation) -> Vec<egui::epaint::RectShape> {
        let shapes = PaintCapture::shapes(|painter| Surface::paint(painter, Self::RECT, level, elevation));
        PaintCapture::rects(&shapes)
    }
}

#[test]
fn floating_has_strong_border_and_shadow() {
    let rects = SurfacePaint::rects(SurfaceLevel::Raised, Elevation::Floating);
    assert!(rects.iter().any(|r| r.fill == color::Shadow::VALUE), "no floating shadow: {rects:?}");
    assert!(rects.iter().any(|r| r.fill == color::BgRaised::VALUE), "no raised fill");
    assert!(rects.iter().any(|r| r.stroke.color == color::LineStrong::VALUE), "no strong border");
    let frame = Surface::frame(SurfaceLevel::Raised, Elevation::Floating);
    assert_eq!(frame.fill, color::BgRaised::VALUE);
    assert_eq!(frame.stroke.color, color::LineStrong::VALUE);
    assert_eq!(frame.shadow, Elevation::Floating.shadow());
}

#[test]
fn flat_has_line_border_and_no_shadow() {
    let rects = SurfacePaint::rects(SurfaceLevel::Panel, Elevation::Flat);
    assert!(rects.iter().all(|r| r.fill != color::Shadow::VALUE), "a flat surface has a shadow");
    assert!(rects.iter().any(|r| r.fill == color::BgPanel::VALUE), "no panel fill");
    assert!(rects.iter().any(|r| r.stroke.color == color::Line::VALUE), "no line border");
    let frame = Surface::frame(SurfaceLevel::Panel, Elevation::Flat);
    assert_eq!(frame.stroke.color, color::Line::VALUE);
    assert_eq!(frame.shadow, Elevation::Flat.shadow());
}

#[test]
fn card_keeps_its_line_border_when_floating() {
    // the Card frame sheet: a translucent card over the map, `--color-line`, with the floating shadow
    let rects = SurfacePaint::rects(SurfaceLevel::Card, Elevation::Floating);
    assert!(rects.iter().any(|r| r.fill == color::Shadow::VALUE));
    assert!(rects.iter().any(|r| r.fill == color::Card::VALUE));
    assert!(rects.iter().any(|r| r.stroke.color == color::Line::VALUE));
    assert!(rects.iter().all(|r| r.stroke.color != color::LineStrong::VALUE));
}

#[test]
fn pill_is_round_with_no_border() {
    let pill = Rect::from_min_size(pos2(20.0, 20.0), vec2(160.0, 28.0));
    let shapes = PaintCapture::shapes(|painter| Surface::paint(painter, pill, SurfaceLevel::Pill, Elevation::Flat));
    let rects = PaintCapture::rects(&shapes);
    let fill = rects.iter().find(|r| r.fill == color::Pill::VALUE).expect("no pill fill");
    assert_eq!(fill.corner_radius.nw, 14);
    assert!(rects.iter().all(|r| r.stroke.width == 0.0 || r.stroke.color.a() == 0), "a pill has a border");
    let frame = Surface::frame(SurfaceLevel::Pill, Elevation::Flat);
    assert_eq!(frame.stroke.width, 0.0);
}
