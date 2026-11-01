use amml_ui::tokens::rule::{Elevation, FocusRing, SoftFill, UnderEdge};
use amml_ui::tokens::{ColorToken, color};
use egui::epaint::Shadow;
use egui::{Color32, CornerRadius, Rect, Shape, pos2};

use crate::support::PaintCapture;

#[test]
fn focus_ring_paints_ring_then_gap() {
    let rect = Rect::from_min_max(pos2(40.0, 40.0), pos2(100.0, 70.0));
    let shapes = PaintCapture::shapes(|painter| FocusRing::paint(painter, rect, CornerRadius::same(6)));
    let rects: Vec<_> =
        shapes.iter().filter_map(|s| if let Shape::Rect(r) = s { Some(r.clone()) } else { None }).collect();
    assert_eq!(rects.len(), 2, "{shapes:?}");
    assert_eq!(rects[0].fill, <color::FocusRing as ColorToken>::VALUE);
    assert_eq!(rects[0].rect, rect.expand(4.0));
    assert_eq!(rects[1].fill, <color::BgPanel as ColorToken>::VALUE);
    assert_eq!(rects[1].rect, rect.expand(2.0));
}

#[test]
fn soft_fill_of_success_is_14_percent() {
    let soft = SoftFill::of(Color32::from_rgb(0x4F, 0xD1, 0xA5));
    assert_eq!(soft.a(), 36);
    assert_eq!(soft, Color32::from_rgba_unmultiplied(0x4F, 0xD1, 0xA5, 36));
}

#[test]
fn focus_ring_has_gap_then_ring() {
    let bg = <color::BgPanel as ColorToken>::VALUE;
    let [gap, ring] = FocusRing::shadows(bg);
    assert_eq!(gap.spread, 2);
    assert_eq!(gap.color, bg);
    assert_eq!(ring.spread, 4);
    assert_eq!(ring.color, <color::AccentHover as ColorToken>::VALUE);
    for shadow in [gap, ring] {
        assert_eq!(shadow.blur, 0);
        assert_eq!(shadow.offset, [0, 0]);
    }
}

#[test]
fn floating_is_0_8_24() {
    let shadow = Elevation::Floating.shadow();
    assert_eq!(shadow.offset, [0, 8]);
    assert_eq!(shadow.blur, 24);
    assert_eq!(shadow.spread, 0);
    assert_eq!(shadow.color, <color::Shadow as ColorToken>::VALUE);
}

#[test]
fn under_edge_is_one_flat_shadow_below() {
    let shadow = UnderEdge::shadow(2);
    assert_eq!(shadow.offset, [0, 2]);
    assert_eq!(shadow.blur, 0);
    assert_eq!(shadow.spread, 0);
    assert_eq!(shadow.color, <color::Shadow as ColorToken>::VALUE);
    assert_eq!(UnderEdge::shadow(3).offset, [0, 3]);
}

#[test]
fn flat_has_no_shadow() {
    assert_eq!(Elevation::Flat.shadow(), Shadow::NONE);
}
