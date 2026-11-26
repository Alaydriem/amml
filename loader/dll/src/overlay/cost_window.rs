use std::time::{Duration, Instant};

use super::CostLine;

/// The present times of one `perf` window. A window closes, and gives its line, at the first frame that makes it at
/// least `MIN_FRAMES` frames and `MIN_SPAN` long; the next window starts there.
#[derive(Debug, Default)]
pub struct CostWindow {
    started: Option<Instant>,
    frames: u64,
    before: Duration,
    before_max: Duration,
    surface: Duration,
    surface_max: Duration,
    unasked: u64,
    asked: u64,
}

impl CostWindow {
    pub const MIN_FRAMES: u64 = 600;
    pub const MIN_SPAN: Duration = Duration::from_secs(10);

    pub const fn new() -> Self {
        Self {
            started: None,
            frames: 0,
            before: Duration::ZERO,
            before_max: Duration::ZERO,
            surface: Duration::ZERO,
            surface_max: Duration::ZERO,
            unasked: 0,
            asked: 0,
        }
    }

    /// Adds one frame that ended at `now`: `before_present` took `before`, its surface part `surface`; `unasked` and
    /// `asked` are the surfaces it made with no request and on request. Gives the line of a window that closes.
    pub fn note(
        &mut self,
        now: Instant,
        before: Duration,
        surface: Duration,
        unasked: u64,
        asked: u64,
    ) -> Option<CostLine> {
        let started = *self.started.get_or_insert(now);
        self.frames += 1;
        self.before += before;
        self.before_max = self.before_max.max(before);
        self.surface += surface;
        self.surface_max = self.surface_max.max(surface);
        self.unasked += unasked;
        self.asked += asked;
        if self.frames < Self::MIN_FRAMES || now.saturating_duration_since(started) < Self::MIN_SPAN {
            return None;
        }
        let line = CostLine {
            frames: self.frames,
            before_us_mean: Self::micros(self.before) / self.frames,
            before_us_max: Self::micros(self.before_max),
            surface_us_mean: Self::micros(self.surface) / self.frames,
            surface_us_max: Self::micros(self.surface_max),
            unasked: self.unasked,
            asked: self.asked,
        };
        *self = Self { started: Some(now), ..Self::new() };
        Some(line)
    }

    fn micros(duration: Duration) -> u64 {
        u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
    }
}
