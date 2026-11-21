use crate::structs::{ClipKind, ClipStep};

/// The cursor clip requests of mods and whether the loader holds a clip now.
///
/// Each mod has at most one request. The last request made wins; the requests before it are kept and come back
/// when the last one drops. A clip is wanted only while the window is active (foreground and not minimised): a
/// loss of that state releases the clip and keeps the requests, a gain applies it again. Every change gives the
/// step the window must take, so the system clip is touched only when it must change.
pub struct ClipBook {
    // Oldest request first; one entry per mod.
    requests: Vec<(usize, ClipKind)>,
    active: bool,
    applied: Option<ClipKind>,
}

impl Default for ClipBook {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipBook {
    pub fn new() -> Self {
        Self { requests: Vec::new(), active: false, applied: None }
    }

    /// Sets the request of `owner` (it becomes the last one), or drops it with `None`.
    pub fn request(&mut self, owner: usize, clip: Option<ClipKind>) -> ClipStep {
        self.requests.retain(|(held, _)| *held != owner);
        if let Some(clip) = clip {
            self.requests.push((owner, clip));
        }
        self.settle(false)
    }

    /// Drops the request of `owner`.
    pub fn end_owner(&mut self, owner: usize) -> ClipStep {
        self.request(owner, None)
    }

    /// The window is active (`true`) or not. A gain applies the wanted clip again at the new centre.
    pub fn set_active(&mut self, active: bool) -> ClipStep {
        self.active = active;
        self.settle(active)
    }

    /// The window changed size or place while active: the wanted clip is applied again at the new centre.
    pub fn moved(&mut self) -> ClipStep {
        self.settle(true)
    }

    /// The window is gone: the clip is given back, the requests stay, and no clip is wanted until the next
    /// `set_active(true)`.
    pub fn window_gone(&mut self) -> ClipStep {
        self.set_active(false)
    }

    // The step to reach the wanted clip. `again` makes a clip that is set be applied once more (a new centre).
    fn settle(&mut self, again: bool) -> ClipStep {
        let wanted = if self.active { self.requests.last().map(|(_, clip)| *clip) } else { None };
        match (wanted, self.applied) {
            (Some(clip), held) if again || held != Some(clip) => {
                self.applied = Some(clip);
                ClipStep::Apply(clip)
            }
            (None, Some(_)) => {
                self.applied = None;
                ClipStep::Release
            }
            _ => ClipStep::Keep,
        }
    }
}
