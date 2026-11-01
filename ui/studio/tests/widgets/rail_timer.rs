use amml_studio_ui::widgets::{RailTimer, RailTimerState};

use super::headless::Headless;

#[test]
fn rail_timer_formats_minutes_and_hours() {
    assert_eq!(RailTimer::label(761_000), "12:41");
    assert_eq!(RailTimer::label(5_000), "00:05");
    assert_eq!(RailTimer::label(3_723_000), "1:02:03");
}

#[test]
fn rail_timer_is_as_wide_as_a_rail_tile() {
    let mut headless = Headless::new();

    let rect = headless.rect(|ui| RailTimer::show(ui, RailTimerState::Paused, 761_000));

    assert_eq!(rect.width(), amml_studio_ui::widgets::RailTile::SIDE);
}
