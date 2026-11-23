use std::time::{Duration, Instant};

use amml::{CostLine, CostWindow};

fn micros(us: u64) -> Duration {
    Duration::from_micros(us)
}

#[test]
fn a_window_closes_at_600_frames_and_10_seconds_with_means_and_maxima() {
    let start = Instant::now();
    let mut window = CostWindow::new();
    let step = Duration::from_millis(20);
    let mut closed = Vec::new();
    for frame in 0..CostWindow::MIN_FRAMES {
        let now = start + step * u32::try_from(frame).unwrap();
        // Every 100th frame makes a surface nobody asked for and is slow; the others cost 100 us with no surface.
        let slow = frame % 100 == 0;
        let (before, surface, unasked) =
            if slow { (micros(1_000), micros(400), 1) } else { (micros(100), micros(0), 0) };
        closed.extend(window.note(now, before, surface, unasked, 0));
    }

    let frames = CostWindow::MIN_FRAMES;
    assert_eq!(
        closed,
        vec![CostLine {
            frames,
            before_us_mean: (6 * 1_000 + 594 * 100) / frames,
            before_us_max: 1_000,
            surface_us_mean: 6 * 400 / frames,
            surface_us_max: 400,
            unasked: 6,
            asked: 0,
        }]
    );
}

#[test]
fn a_window_waits_for_10_seconds_when_frames_come_fast() {
    let start = Instant::now();
    let mut window = CostWindow::new();
    for frame in 0..CostWindow::MIN_FRAMES * 2 {
        let now = start + Duration::from_millis(frame);
        assert_eq!(window.note(now, micros(50), micros(0), 0, 1), None);
    }

    let line = window.note(start + CostWindow::MIN_SPAN, micros(50), micros(10), 0, 1).unwrap();

    assert_eq!(line.frames, CostWindow::MIN_FRAMES * 2 + 1);
    assert_eq!(line.asked, CostWindow::MIN_FRAMES * 2 + 1);
    assert_eq!(line.before_us_mean, 50);
    // The next window starts empty.
    assert_eq!(window.note(start + CostWindow::MIN_SPAN * 2, micros(50), micros(0), 0, 0), None);
}
