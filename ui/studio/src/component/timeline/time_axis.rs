/// Seconds to x and back for the timeline's zoom and scroll: `start` is the time at x `left`, and each point is
/// `seconds_per_point` seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeAxis {
    pub start: f64,
    pub seconds_per_point: f64,
    pub left: f32,
}

impl TimeAxis {
    pub fn x_of(&self, seconds: f64) -> f32 {
        (f64::from(self.left) + (seconds - self.start) / self.seconds_per_point) as f32
    }

    pub fn seconds_at(&self, x: f32) -> f64 {
        self.start + f64::from(x - self.left) * self.seconds_per_point
    }
}
