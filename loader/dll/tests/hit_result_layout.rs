use std::mem::offset_of;

use amml::{AabbShapeLayout, HitResultLayout};

#[test]
fn layout_size_matches_the_survey() {
    assert_eq!(size_of::<HitResultLayout>(), 0x88);
    assert_eq!(offset_of!(HitResultLayout, kind), 0x18);
    assert_eq!(offset_of!(HitResultLayout, facing), 0x1C);
    assert_eq!(offset_of!(HitResultLayout, block), 0x20);
    assert_eq!(offset_of!(HitResultLayout, pos), 0x2C);
    assert_eq!(offset_of!(HitResultLayout, weak_control), 0x40);
    assert_eq!(offset_of!(HitResultLayout, entity_box), 0x50);
    assert_eq!(offset_of!(HitResultLayout, indirect_hit), 0x84);
}

#[test]
fn the_box_is_the_first_member_of_the_shape_component() {
    assert_eq!(offset_of!(AabbShapeLayout, aabb), 0);
    assert_eq!(size_of::<AabbShapeLayout>(), 0x20);
}
