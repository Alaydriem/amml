use amml_abi::AmmlLightFalloff;
use amml_runtime::{LightBook, LightPoint, LightRefusal};

fn light(x: f32, level: u8) -> Option<LightPoint> {
    LightPoint::at([x, 64.5, 0.5], level, AmmlLightFalloff::ROUND, 0)
}

fn quiet(_: &[LightPoint]) {}

#[test]
fn a_new_light_rebuilds_its_reach() {
    let book = LightBook::new();
    let change = book.set(1, light(0.5, 3), quiet).unwrap();
    assert_eq!(change.boxes, vec![([-3, 61, -3], [3, 67, 3])]);
}

#[test]
fn a_move_rebuilds_the_old_and_the_new_reach() {
    let book = LightBook::new();
    book.set(1, light(0.5, 15), quiet).unwrap();
    let change = book.set(1, light(1.5, 3), quiet).unwrap();
    assert_eq!(change.boxes, vec![([-15, 49, -15], [15, 79, 15]), ([-2, 61, -3], [4, 67, 3])]);
}

#[test]
fn the_same_light_changes_nothing_and_publishes_nothing() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    let mut published = false;
    let change = book.set(1, light(0.6, 3), |_| published = true).unwrap();
    assert!(change.is_empty());
    assert!(!published);
}

#[test]
fn a_falloff_change_rebuilds_without_a_move() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    let taxicab = LightPoint::at([0.5, 64.5, 0.5], 3, AmmlLightFalloff::TAXICAB, 0);
    assert_eq!(book.set(1, taxicab, quiet).unwrap().boxes.len(), 1);
}

#[test]
fn clear_rebuilds_the_old_reach() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    let mut seen = None;
    let change = book.set(1, None, |points| seen = Some(points.len())).unwrap();
    assert_eq!(change.boxes, vec![([-3, 61, -3], [3, 67, 3])]);
    assert_eq!(seen, Some(0));
    assert!(!book.holds(1));
    assert!(book.set(1, None, quiet).unwrap().is_empty());
}

#[test]
fn fifth_slot_is_refused() {
    let book = LightBook::new();
    for owner in 0..4 {
        book.set(owner, light(owner as f32, 3), quiet).unwrap();
    }
    assert_eq!(book.set(9, light(0.5, 3), quiet), Err(LightRefusal::Full));
    assert_eq!(book.points().len(), 4);
    assert!(book.set(2, light(40.0, 5), quiet).is_ok());
}

#[test]
fn disable_one_of_two_keeps_the_other() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    book.set(2, light(100.5, 5), quiet).unwrap();
    let change = book.set(1, None, quiet).unwrap();
    assert_eq!(change.boxes, vec![([-3, 61, -3], [3, 67, 3])]);
    assert_eq!(book.points(), vec![light(100.5, 5).unwrap()]);
}

#[test]
fn clear_all_rebuilds_every_reach() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    book.set(2, light(100.5, 5), quiet).unwrap();
    assert_eq!(book.clear_all(quiet).boxes.len(), 2);
    assert!(book.points().is_empty());
}
