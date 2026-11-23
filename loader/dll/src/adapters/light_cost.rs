use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use crate::LightCostLine;

/// What the held light costs: light changes, the subchunks they mark and the `_setDirty` calls that mark them,
/// the light evaluations that found the marks, and the light getter calls while a light is set (all, and those
/// inside the reach of the lights). Counted only when on (`AMML_PERF=1` at install); a window closes, and gives
/// its `perf` line, at the first frame at least `MIN_SPAN` after it started.
pub struct LightCost {
    on: AtomicBool,
    changes: AtomicU64,
    evaluations: AtomicU64,
    subchunks: AtomicU64,
    set_dirty: AtomicU64,
    lit_getters: AtomicU64,
    near_getters: AtomicU64,
    started: Mutex<Option<Instant>>,
}

impl LightCost {
    /// The variable that turns the counts on, with the value `1`.
    pub const ENV: &'static str = "AMML_PERF";
    pub const MIN_SPAN: Duration = Duration::from_secs(10);

    pub const fn new(on: bool) -> Self {
        Self {
            on: AtomicBool::new(on),
            changes: AtomicU64::new(0),
            evaluations: AtomicU64::new(0),
            subchunks: AtomicU64::new(0),
            set_dirty: AtomicU64::new(0),
            lit_getters: AtomicU64::new(0),
            near_getters: AtomicU64::new(0),
            started: Mutex::new(None),
        }
    }

    /// On when `AMML_PERF` is `1` in the environment now.
    pub fn from_env() -> Self {
        Self::new(std::env::var(Self::ENV).is_ok_and(|value| value == "1"))
    }

    pub fn is_on(&self) -> bool {
        self.on.load(Ordering::Relaxed)
    }

    /// A light change that took `evaluations` light evaluations to find its marks.
    pub fn note_change(&self, evaluations: u64) {
        if self.is_on() {
            self.changes.fetch_add(1, Ordering::Relaxed);
            self.evaluations.fetch_add(evaluations, Ordering::Relaxed);
        }
    }

    /// A rebuild marked `subchunks` subchunks with `calls` `_setDirty` calls.
    pub fn note_marks(&self, subchunks: usize, calls: usize) {
        if self.is_on() {
            self.subchunks.fetch_add(u64::try_from(subchunks).unwrap_or(u64::MAX), Ordering::Relaxed);
            self.set_dirty.fetch_add(u64::try_from(calls).unwrap_or(u64::MAX), Ordering::Relaxed);
        }
    }

    /// A light getter call while a light is set; `near` when its position is inside the reach of the lights.
    pub fn note_getter(&self, near: bool) {
        if self.is_on() {
            self.lit_getters.fetch_add(1, Ordering::Relaxed);
            if near {
                self.near_getters.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Once per frame at `now`: the line of a window that closes; the next window starts there.
    pub fn take_line(&self, now: Instant) -> Option<LightCostLine> {
        if !self.is_on() {
            return None;
        }
        let mut started = self.started.lock().unwrap_or_else(PoisonError::into_inner);
        let since = *started.get_or_insert(now);
        let span = now.saturating_duration_since(since);
        if span < Self::MIN_SPAN {
            return None;
        }
        *started = Some(now);
        let span_ms = u64::try_from(span.as_millis()).unwrap_or(u64::MAX).max(1);
        let rate = |count: &AtomicU64| count.swap(0, Ordering::Relaxed).saturating_mul(1000) / span_ms;
        Some(LightCostLine {
            span_ms,
            changes_per_s: rate(&self.changes),
            subchunks_per_s: rate(&self.subchunks),
            set_dirty_per_s: rate(&self.set_dirty),
            evaluations_per_s: rate(&self.evaluations),
            lit_getters_per_s: rate(&self.lit_getters),
            near_getters_per_s: rate(&self.near_getters),
        })
    }
}
