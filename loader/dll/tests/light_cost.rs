use std::time::{Duration, Instant};

use amml::{LightCost, LightCostLine};

#[test]
fn off_counts_nothing_and_gives_no_line() {
    let cost = LightCost::new(false);
    let start = Instant::now();
    cost.note_change(10);
    cost.note_getter(true);
    assert_eq!(cost.take_line(start), None);
    assert_eq!(cost.take_line(start + Duration::from_secs(20)), None);
}

#[test]
fn a_window_gives_per_second_rates_once_per_span() {
    let cost = LightCost::new(true);
    let start = Instant::now();
    assert_eq!(cost.take_line(start), None);
    for _ in 0..170 {
        cost.note_change(1700);
    }
    cost.note_marks(3500, 1200);
    for index in 0..1000 {
        cost.note_getter(index % 4 == 0);
    }
    assert_eq!(cost.take_line(start + Duration::from_secs(9)), None);
    let line = cost.take_line(start + Duration::from_secs(10));
    assert_eq!(
        line,
        Some(LightCostLine {
            span_ms: 10_000,
            changes_per_s: 17,
            subchunks_per_s: 350,
            set_dirty_per_s: 120,
            evaluations_per_s: 28_900,
            lit_getters_per_s: 100,
            near_getters_per_s: 25,
        })
    );
    // The next window starts at the line and counts from 0.
    assert_eq!(cost.take_line(start + Duration::from_secs(15)), None);
    let next = cost.take_line(start + Duration::from_secs(20)).unwrap();
    assert_eq!((next.changes_per_s, next.lit_getters_per_s), (0, 0));
}
