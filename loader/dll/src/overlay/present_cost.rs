use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use super::{CostWindow, SurfaceCounts};

/// What the overlay's presents cost. The surface counts are always kept (one relaxed add per surface). The times are
/// taken only when `AMML_PERF=1` was set at the claim; they go to a `perf` line per `CostWindow`.
pub struct PresentCost {
    timed: AtomicBool,
    made: AtomicU64,
    asked: AtomicU64,
    state_saves: AtomicU64,
    // The surfaces of the frame in progress, made with no request and on request, and its surface time.
    frame_unasked: AtomicU64,
    frame_asked: AtomicU64,
    frame_surface_ns: AtomicU64,
    window: Mutex<CostWindow>,
}

impl PresentCost {
    /// The variable that turns the times on, with the value `1`.
    pub const ENV: &'static str = "AMML_PERF";

    pub const fn new() -> Self {
        Self {
            timed: AtomicBool::new(false),
            made: AtomicU64::new(0),
            asked: AtomicU64::new(0),
            state_saves: AtomicU64::new(0),
            frame_unasked: AtomicU64::new(0),
            frame_asked: AtomicU64::new(0),
            frame_surface_ns: AtomicU64::new(0),
            window: Mutex::new(CostWindow::new()),
        }
    }

    /// Takes the times when `AMML_PERF` is `1` in the environment now.
    pub fn read_env(&self) {
        let timed = std::env::var(Self::ENV).is_ok_and(|value| value == "1");
        self.timed.store(timed, Ordering::Relaxed);
    }

    /// The start of a timed part; `None` while the times are off.
    pub fn clock(&self) -> Option<Instant> {
        self.timed.load(Ordering::Relaxed).then(Instant::now)
    }

    /// A surface was made, on a listener's request when `asked`; its surface part began at `since`.
    pub fn note_surface(&self, asked: bool, since: Option<Instant>) {
        self.made.fetch_add(1, Ordering::Relaxed);
        if asked {
            self.asked.fetch_add(1, Ordering::Relaxed);
            self.frame_asked.fetch_add(1, Ordering::Relaxed);
        } else {
            self.frame_unasked.fetch_add(1, Ordering::Relaxed);
        }
        self.add_surface_time(since);
    }

    /// A frame's `before_present` begins: the per-frame counts start from 0, also after a frame that returned early.
    /// Gives the start of its time while the times are on.
    pub fn begin_frame(&self) -> Option<Instant> {
        self.frame_unasked.store(0, Ordering::Relaxed);
        self.frame_asked.store(0, Ordering::Relaxed);
        self.frame_surface_ns.store(0, Ordering::Relaxed);
        self.clock()
    }

    /// A part of the frame's surface setup (the acquire of the wrapped back buffer) began at `since`.
    pub fn note_surface_time(&self, since: Option<Instant>) {
        self.add_surface_time(since);
    }

    /// The pipeline state was saved for the listeners, with the copy window and draw flag; the save began at
    /// `since`.
    pub fn note_state_save(&self, since: Option<Instant>) {
        self.state_saves.fetch_add(1, Ordering::Relaxed);
        self.add_surface_time(since);
    }

    /// The frame whose `before_present` began at `since` ends; logs the `perf` line of a window that closes.
    pub fn end_frame(&self, since: Option<Instant>) {
        let unasked = self.frame_unasked.swap(0, Ordering::Relaxed);
        let asked = self.frame_asked.swap(0, Ordering::Relaxed);
        let surface = Duration::from_nanos(self.frame_surface_ns.swap(0, Ordering::Relaxed));
        let Some(since) = since else {
            return;
        };
        let now = Instant::now();
        let before = now.saturating_duration_since(since);
        let line =
            self.window.lock().unwrap_or_else(PoisonError::into_inner).note(now, before, surface, unasked, asked);
        if let Some(line) = line {
            line.log();
        }
    }

    pub fn counts(&self) -> SurfaceCounts {
        SurfaceCounts {
            made: self.made.load(Ordering::Relaxed),
            asked: self.asked.load(Ordering::Relaxed),
            state_saves: self.state_saves.load(Ordering::Relaxed),
        }
    }

    fn add_surface_time(&self, since: Option<Instant>) {
        if let Some(since) = since {
            let spent = u64::try_from(since.elapsed().as_nanos()).unwrap_or(u64::MAX);
            self.frame_surface_ns.fetch_add(spent, Ordering::Relaxed);
        }
    }
}

impl Default for PresentCost {
    fn default() -> Self {
        Self::new()
    }
}
