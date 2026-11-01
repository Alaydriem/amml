use amml_studio_ui::component::coverage::{BadgePlacement, CoverageBadge, PlacedBadge};
use amml_studio_ui::tokens::selector::StatusColor;
use egui::{Rect, pos2, vec2};

use crate::glyph::words;
use crate::support::PaintCapture;

fn placed(placement: BadgePlacement) -> Rect {
    let control = Rect::from_min_size(pos2(100.0, 100.0), vec2(140.0, 34.0));
    let badge = PlacedBadge::new(StatusColor::Planned, words("G11 PLANNED"), placement);
    let mut rect = Rect::NOTHING;
    PaintCapture::shapes(|painter| rect = badge.paint_on(painter, control));
    rect
}

#[test]
fn placed_badge_sits_beside_above_or_below_the_control() {
    let control = Rect::from_min_size(pos2(100.0, 100.0), vec2(140.0, 34.0));
    let right = placed(BadgePlacement::Right);
    assert!(right.left() > control.right() && (right.center().y - control.center().y).abs() < 0.6, "{right:?}");
    let above = placed(BadgePlacement::Above);
    assert!(above.bottom() < control.top() && (above.left() - control.left()).abs() < 0.01, "{above:?}");
    let below = placed(BadgePlacement::Below);
    assert!(below.top() > control.bottom() && (below.left() - control.left()).abs() < 0.01, "{below:?}");
}

#[test]
fn placed_badge_keeps_the_gap_to_the_control() {
    let control = Rect::from_min_size(pos2(100.0, 100.0), vec2(140.0, 34.0));
    let above = placed(BadgePlacement::Above);
    assert!((control.top() - above.bottom() - PlacedBadge::GAP).abs() < 0.01);
    assert!((above.height() - CoverageBadge::HEIGHT).abs() < 0.01);
}
