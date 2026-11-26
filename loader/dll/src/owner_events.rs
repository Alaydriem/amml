use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

use amml_abi::{
    AmmlPhase, InjectedInputReleased, InputLeaseLost, InputLeaseResumed, InputLeaseSuspended, JoinServerResult,
    ScreenshotTaken, ScreenshotWritten,
};

use crate::{EventHub, OwnerEvent};

/// Lease notices, screenshot steps and join answers waiting for the next frame, each for one mod.
///
/// Any thread pushes; the game thread drains at `FrameUpdate` Before, so a listener always runs on
/// the game thread and never inside the call that ended the lease.
#[derive(Debug, Default)]
pub struct OwnerEvents {
    queue: Mutex<VecDeque<(usize, OwnerEvent)>>,
}

impl OwnerEvents {
    pub fn new() -> Self {
        Self { queue: Mutex::new(VecDeque::new()) }
    }

    /// Queues `event` for the mod at `owner`.
    pub fn push(&self, owner: usize, event: OwnerEvent) {
        self.lock().push_back((owner, event));
    }

    /// How many notices wait for the mod at `owner`.
    pub fn queued_for(&self, owner: usize) -> usize {
        self.lock().iter().filter(|(queued, _)| *queued == owner).count()
    }

    /// Raises every queued notice to its owner's listeners, in the order pushed. The queue is taken
    /// under the lock and raised outside it, so a listener may push.
    pub fn drain(&self, hub: &EventHub) {
        let queued = std::mem::take(&mut *self.lock());
        let context = hub.loader_context();
        for (owner, event) in queued {
            match event {
                OwnerEvent::Suspended { lease } => {
                    let header = EventHub::header::<InputLeaseSuspended>(AmmlPhase::AFTER);
                    hub.raise_only(&mut InputLeaseSuspended { header, lease }, &context, owner);
                }
                OwnerEvent::Resumed { lease } => {
                    let header = EventHub::header::<InputLeaseResumed>(AmmlPhase::AFTER);
                    hub.raise_only(&mut InputLeaseResumed { header, lease }, &context, owner);
                }
                OwnerEvent::Lost { lease, reason } => {
                    let header = EventHub::header::<InputLeaseLost>(AmmlPhase::AFTER);
                    hub.raise_only(&mut InputLeaseLost { header, lease, reason }, &context, owner);
                }
                OwnerEvent::Released { lease, reason } => {
                    let header = EventHub::header::<InjectedInputReleased>(AmmlPhase::AFTER);
                    let input = InjectedInputReleased::ALL;
                    let mut event = InjectedInputReleased { header, lease, reason, input, code: 0, _reserved: 0 };
                    hub.raise_only(&mut event, &context, owner);
                }
                OwnerEvent::ReleasedOne { lease, reason, input, code } => {
                    let header = EventHub::header::<InjectedInputReleased>(AmmlPhase::AFTER);
                    let mut event = InjectedInputReleased { header, lease, reason, input, code, _reserved: 0 };
                    hub.raise_only(&mut event, &context, owner);
                }
                OwnerEvent::ScreenshotTaken { id, ok } => {
                    let header = EventHub::header::<ScreenshotTaken>(AmmlPhase::AFTER);
                    let mut event = ScreenshotTaken { header, id, ok: u8::from(ok), _reserved: [0; 7] };
                    hub.raise_only(&mut event, &context, owner);
                }
                OwnerEvent::ScreenshotWritten { id, ok } => {
                    let header = EventHub::header::<ScreenshotWritten>(AmmlPhase::AFTER);
                    let mut event = ScreenshotWritten { header, id, ok: u8::from(ok), _reserved: [0; 7] };
                    hub.raise_only(&mut event, &context, owner);
                }
                OwnerEvent::JoinServerResult { join, outcome } => {
                    let header = EventHub::header::<JoinServerResult>(AmmlPhase::AFTER);
                    let mut event = JoinServerResult { header, join, outcome, _reserved: 0 };
                    hub.raise_only(&mut event, &context, owner);
                }
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<(usize, OwnerEvent)>> {
        self.queue.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
