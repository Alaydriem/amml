use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use amml_abi::AmmlShotLayers;

use crate::structs::{CapturePoint, ShotRequest};

struct Pending {
    point: CapturePoint,
    request: ShotRequest,
    // The present count and the time when the request was made.
    asked_present: u64,
    asked_at: Instant,
}

struct BookState {
    pending: Vec<Pending>,
    next_id: u64,
}

/// Screenshot requests waiting for their capture point. Each is handed out once: by `due` when its point comes, by
/// `expired` when it waited too long, or by `take_all`.
pub struct ShotBook {
    state: Mutex<BookState>,
}

impl Default for ShotBook {
    fn default() -> Self {
        Self::new()
    }
}

impl ShotBook {
    pub fn new() -> Self {
        Self { state: Mutex::new(BookState { pending: Vec::new(), next_id: 1 }) }
    }

    fn lock(&self) -> MutexGuard<'_, BookState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Queues a screenshot asked at present count `present` and time `now`, and gives its id. `GameOnly` is taken
    /// before the listeners; any other value takes the image with overlays, after the normal band.
    pub fn request(&self, owner: usize, path: PathBuf, layers: AmmlShotLayers, present: u64, now: Instant) -> u64 {
        let mut state = self.lock();
        let id = state.next_id;
        state.next_id += 1;
        let point = if layers == AmmlShotLayers::GAME_ONLY {
            CapturePoint::BeforeListeners
        } else {
            CapturePoint::AfterNormalBand
        };
        state.pending.push(Pending {
            point,
            request: ShotRequest { id, owner, path },
            asked_present: present,
            asked_at: now,
        });
        id
    }

    /// Takes the requests for `point`, oldest first.
    pub fn due(&self, point: CapturePoint) -> Vec<ShotRequest> {
        let mut state = self.lock();
        let (due, kept): (Vec<_>, Vec<_>) = state.pending.drain(..).partition(|pending| pending.point == point);
        state.pending = kept;
        due.into_iter().map(|pending| pending.request).collect()
    }

    /// Takes the requests that waited at least `presents` presents or at least `wait`, at present count `present` and
    /// time `now`, oldest first.
    pub fn expired(&self, present: u64, now: Instant, presents: u64, wait: Duration) -> Vec<ShotRequest> {
        let mut state = self.lock();
        let (expired, kept): (Vec<_>, Vec<_>) = state.pending.drain(..).partition(|pending| {
            present.saturating_sub(pending.asked_present) >= presents
                || now.saturating_duration_since(pending.asked_at) >= wait
        });
        state.pending = kept;
        expired.into_iter().map(|pending| pending.request).collect()
    }

    /// Takes every waiting request, oldest first.
    pub fn take_all(&self) -> Vec<ShotRequest> {
        self.lock().pending.drain(..).map(|pending| pending.request).collect()
    }

    /// True when no request waits for a capture point.
    pub fn is_empty(&self) -> bool {
        self.lock().pending.is_empty()
    }

    /// Forgets the requests of `owner`.
    pub fn drop_owner(&self, owner: usize) {
        self.lock().pending.retain(|pending| pending.request.owner != owner);
    }
}
