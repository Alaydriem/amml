/// What one held timer hands out at an advance: the whole ticks since its last advance and the alpha (the fraction
/// of a tick past the last whole tick).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimerReading {
    pub ticks: i32,
    pub alpha: f32,
}
