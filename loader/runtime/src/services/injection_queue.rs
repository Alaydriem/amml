use std::sync::{Mutex, MutexGuard};

use amml_abi::{AmmlMouseButton, AmmlMoveIntent, AmmlReleaseReason};

use crate::structs::{
    FrameInjection, InjectedOp, InjectionState, LeaseInput, LeaseNotice, NoticeKind, ReleaseOutcome, ResourceId,
};

/// Holds injected input until the next frame applies it.
///
/// A key or button is held from its down to its up: a second down of a held
/// key and an up of a key that is not held are dropped, so the game gets one
/// down and one up per hold. `release` queues one up for each key and button
/// the game holds and drops the lease's queued key and button ops; a down that
/// is dropped before a frame takes it reaches the game as nothing, neither the
/// down nor an up. A release also drops the lease's queued wheel ops and its
/// turn not yet taken.
pub struct InjectionQueue {
    state: Mutex<InjectionState>,
}

impl Default for InjectionQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl InjectionQueue {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(InjectionState {
                pending: Vec::new(),
                inputs: Vec::new(),
                turns: Vec::new(),
                move_intent: None,
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, InjectionState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Queues a key down or up of the lease. False when it was dropped: a down
    /// of a key the lease holds, or an up of a key it does not hold.
    pub fn key(&self, lease: ResourceId, key: u16, down: bool) -> bool {
        let mut state = self.lock();
        let input = Self::input_of(&mut state, lease);
        let queued = Self::toggle(&mut input.wanted_keys, key, down);
        if queued {
            state.pending.push((lease, InjectedOp::Key { key, down }));
        }
        queued
    }

    /// Queues a button down or up of the lease, under the rules of `key`.
    pub fn button(&self, lease: ResourceId, button: AmmlMouseButton, down: bool) -> bool {
        let mut state = self.lock();
        let input = Self::input_of(&mut state, lease);
        let queued = Self::toggle(&mut input.wanted_buttons, button, down);
        if queued {
            state.pending.push((lease, InjectedOp::Button { button, down }));
        }
        queued
    }

    /// Queues the down `op` of a key or button the lease already holds, for a
    /// hold that a real up ended in the game. Nothing for an up, a wheel op, or
    /// a code the lease does not hold.
    pub fn press_again(&self, lease: ResourceId, op: InjectedOp) {
        let mut state = self.lock();
        let Some(input) = state.inputs.iter().find(|input| input.lease == lease) else {
            return;
        };
        let held = match op {
            InjectedOp::Key { key, down: true } => input.wanted_keys.contains(&key),
            InjectedOp::Button { button, down: true } => input.wanted_buttons.contains(&button),
            _ => false,
        };
        if held {
            state.pending.push((lease, op));
        }
    }

    /// Forgets that the lease holds the key or button of the down `op`, for a
    /// hold that a real up ended in the game: no up is queued for it, and a
    /// later down is queued as a new press. Nothing for an up or a wheel op.
    pub fn forget(&self, lease: ResourceId, op: InjectedOp) {
        let mut state = self.lock();
        let Some(input) = state.inputs.iter_mut().find(|input| input.lease == lease) else {
            return;
        };
        match op {
            InjectedOp::Key { key, down: true } => {
                input.wanted_keys.retain(|held| *held != key);
                input.applied_keys.retain(|held| *held != key);
            }
            InjectedOp::Button { button, down: true } => {
                input.wanted_buttons.retain(|held| *held != button);
                input.applied_buttons.retain(|held| *held != button);
            }
            _ => {}
        }
    }

    /// Adds wheel notches to the lease's fraction and queues the whole notches
    /// that build up; the rest waits for the next call. A value that is not
    /// finite is dropped.
    pub fn wheel(&self, lease: ResourceId, notches: f32) {
        if !notches.is_finite() {
            return;
        }
        let mut state = self.lock();
        let input = Self::input_of(&mut state, lease);
        let total = input.wheel_rest + notches;
        let whole = total.trunc();
        input.wheel_rest = total - whole;
        if whole != 0.0 {
            state.pending.push((lease, InjectedOp::Wheel { notches: whole }));
        }
    }

    /// Adds a turn to the lease's part of the frame's sum. A value that is not finite is dropped.
    pub fn turn(&self, lease: ResourceId, yaw: f32, pitch: f32) {
        if !yaw.is_finite() || !pitch.is_finite() {
            return;
        }
        let mut state = self.lock();
        match state.turns.iter_mut().find(|(owner, _, _)| *owner == lease) {
            Some((_, sum_yaw, sum_pitch)) => {
                *sum_yaw += yaw;
                *sum_pitch += pitch;
            }
            None => state.turns.push((lease, yaw, pitch)),
        }
    }

    /// Sets or clears the lease's move intent. A new intent from another lease
    /// replaces the current one (`Released(REPLACED)` to its owner); a clear
    /// from a lease that does not own the intent changes nothing.
    pub fn set_move(&self, lease: ResourceId, owner: usize, intent: Option<AmmlMoveIntent>) -> Vec<LeaseNotice> {
        let mut state = self.lock();
        let mut notices = Vec::new();
        match intent {
            Some(intent) => {
                if let Some((current, current_owner, _)) = state.move_intent
                    && current != lease
                {
                    notices.push(LeaseNotice {
                        owner: current_owner,
                        lease: current,
                        kind: NoticeKind::Released(AmmlReleaseReason::REPLACED),
                    });
                }
                state.move_intent = Some((lease, owner, intent));
            }
            None => {
                if state.move_intent.is_some_and(|(current, _, _)| current == lease) {
                    state.move_intent = None;
                }
            }
        }
        notices
    }

    /// Whether the lease owns the move intent.
    pub fn has_move(&self, lease: ResourceId) -> bool {
        self.lock().move_intent.is_some_and(|(current, _, _)| current == lease)
    }

    /// The current move intent, left in place for the next read.
    pub fn move_intent(&self) -> Option<AmmlMoveIntent> {
        self.lock().move_intent.map(|(_, _, intent)| intent)
    }

    /// The keys and buttons the lease holds, as downs: queued or applied, with no up since.
    pub fn held(&self, lease: ResourceId) -> Vec<InjectedOp> {
        let state = self.lock();
        let Some(input) = state.inputs.iter().find(|input| input.lease == lease) else {
            return Vec::new();
        };
        let keys = input.wanted_keys.iter().map(|&key| InjectedOp::Key { key, down: true });
        let buttons = input.wanted_buttons.iter().map(|&button| InjectedOp::Button { button, down: true });
        keys.chain(buttons).collect()
    }

    /// Releases everything the lease holds and its move intent, drops its
    /// queued ops and its turn not yet taken, and forgets its wheel fraction.
    /// Queues one up for each key and button the game holds. `ups` are the ups
    /// of the holds this call ended; an up already queued by an earlier release
    /// or an up of the mod's own is queued again but not given. `had_input` is
    /// read under the same lock, before anything is cleared.
    pub fn release(&self, lease: ResourceId) -> ReleaseOutcome {
        let mut state = self.lock();
        Self::release_locked(&mut state, lease)
    }

    /// Releases every lease, as `release` does. Gives the leases that held
    /// input or got ups, in the order of their first queued input, the move
    /// intent's lease last when it held nothing else.
    pub fn release_everything(&self) -> Vec<(ResourceId, ReleaseOutcome)> {
        let mut state = self.lock();
        let mut leases: Vec<ResourceId> = state.inputs.iter().map(|input| input.lease).collect();
        if let Some((lease, _, _)) = state.move_intent
            && !leases.contains(&lease)
        {
            leases.push(lease);
        }
        let mut released = Vec::new();
        for lease in leases {
            let outcome = Self::release_locked(&mut state, lease);
            if outcome.had_input || !outcome.ups.is_empty() {
                released.push((lease, outcome));
            }
        }
        state.turns.clear();
        released
    }

    /// Takes the queued ops and the summed turn; the move intent stays.
    pub fn take_frame(&self) -> FrameInjection {
        let mut state = self.lock();
        let ops = std::mem::take(&mut state.pending);
        for (lease, op) in &ops {
            let input = Self::input_of(&mut state, *lease);
            match *op {
                InjectedOp::Key { key, down } => {
                    Self::toggle(&mut input.applied_keys, key, down);
                }
                InjectedOp::Button { button, down } => {
                    Self::toggle(&mut input.applied_buttons, button, down);
                }
                InjectedOp::Wheel { .. } => {}
            }
        }
        state.inputs.retain(|input| {
            !(input.wanted_keys.is_empty()
                && input.applied_keys.is_empty()
                && input.wanted_buttons.is_empty()
                && input.applied_buttons.is_empty()
                && input.wheel_rest == 0.0)
        });
        let turn = std::mem::take(&mut state.turns)
            .into_iter()
            .map(|(_, yaw, pitch)| (yaw, pitch))
            .reduce(|(sum_yaw, sum_pitch), (yaw, pitch)| (sum_yaw + yaw, sum_pitch + pitch));
        let move_intent = state.move_intent.map(|(lease, _, intent)| (lease, intent));
        FrameInjection { ops, turn, move_intent }
    }

    fn release_locked(state: &mut InjectionState, lease: ResourceId) -> ReleaseOutcome {
        let had_move = state.move_intent.is_some_and(|(current, _, _)| current == lease);
        if had_move {
            state.move_intent = None;
        }
        state.pending.retain(|(owner, _)| *owner != lease);
        state.turns.retain(|(owner, _, _)| *owner != lease);
        let Some(input) = state.inputs.iter_mut().find(|input| input.lease == lease) else {
            return ReleaseOutcome { ups: Vec::new(), had_input: had_move };
        };
        let had_input = had_move || !input.wanted_keys.is_empty() || !input.wanted_buttons.is_empty();
        let mut queued = Vec::new();
        let mut ended = Vec::new();
        for &key in &input.applied_keys {
            let up = InjectedOp::Key { key, down: false };
            queued.push(up);
            if input.wanted_keys.contains(&key) {
                ended.push(up);
            }
        }
        for &button in &input.applied_buttons {
            let up = InjectedOp::Button { button, down: false };
            queued.push(up);
            if input.wanted_buttons.contains(&button) {
                ended.push(up);
            }
        }
        input.wanted_keys.clear();
        input.wanted_buttons.clear();
        input.wheel_rest = 0.0;
        state.pending.extend(queued.into_iter().map(|op| (lease, op)));
        ReleaseOutcome { ups: ended, had_input }
    }

    fn input_of(state: &mut InjectionState, lease: ResourceId) -> &mut LeaseInput {
        let index = match state.inputs.iter().position(|input| input.lease == lease) {
            Some(index) => index,
            None => {
                state.inputs.push(LeaseInput {
                    lease,
                    wanted_keys: Vec::new(),
                    applied_keys: Vec::new(),
                    wanted_buttons: Vec::new(),
                    applied_buttons: Vec::new(),
                    wheel_rest: 0.0,
                });
                state.inputs.len() - 1
            }
        };
        &mut state.inputs[index]
    }

    // Adds `code` on a down it does not hold, removes it on an up it holds.
    // False when the op changes nothing and must be dropped.
    fn toggle<T: PartialEq>(held: &mut Vec<T>, code: T, down: bool) -> bool {
        let position = held.iter().position(|item| *item == code);
        match (down, position) {
            (true, None) => {
                held.push(code);
                true
            }
            (false, Some(index)) => {
                held.remove(index);
                true
            }
            _ => false,
        }
    }
}
