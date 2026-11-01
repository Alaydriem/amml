use amml_ui::draw::Corner;
use egui::{CornerRadius, Rect, pos2, vec2};

#[test]
fn full_corner_is_half_the_short_side() {
    let toggle = Rect::from_min_size(pos2(0.0, 0.0), vec2(34.0, 20.0));
    assert_eq!(Corner::Full.radius(toggle), CornerRadius::same(10));
    let huge = Rect::from_min_size(pos2(0.0, 0.0), vec2(600.0, 600.0));
    assert_eq!(Corner::Full.radius(huge), CornerRadius::same(255));
}

#[test]
fn named_corners_follow_the_radius_tokens() {
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
    assert_eq!(Corner::None.radius(rect), CornerRadius::ZERO);
    assert_eq!(Corner::Small.radius(rect), CornerRadius::same(4));
    assert_eq!(Corner::Control.radius(rect), CornerRadius::same(6));
    assert_eq!(Corner::Panel.radius(rect), CornerRadius::same(10));
}
