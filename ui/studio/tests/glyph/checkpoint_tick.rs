use amml_studio_ui::glyph::CheckpointTick;
use amml_ui::glyph::Glyph;
use egui::{Color32, Rect, pos2, vec2};

use crate::support::PaintCapture;

const GREY: Color32 = Color32::from_rgba_unmultiplied_const(154, 161, 175, 140);

#[test]
fn checkpoint_tick_stays_in_its_rect() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(20.0, 20.0));
    let shapes = PaintCapture::shapes(|painter| CheckpointTick.paint(painter, rect, GREY));
    assert!(rect.contains_rect(PaintCapture::bounds(&shapes)), "{:?}", PaintCapture::bounds(&shapes));
}

#[test]
fn checkpoint_tick_stands_on_the_bottom_edge_at_the_centre() {
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(20.0, 20.0));
    let shapes = PaintCapture::shapes(|painter| CheckpointTick.paint(painter, rect, GREY));
    let bounds = PaintCapture::bounds(&shapes);
    assert!((bounds.bottom() - rect.bottom()).abs() < 0.01, "{bounds:?}");
    assert!((bounds.center().x - rect.center().x).abs() < 0.01, "{bounds:?}");
}
