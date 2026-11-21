use amml_abi::AmmlRequestOutcome;

use crate::structs::{ActiveRequest, RequestPhase};
use crate::{
    ProtocolValues, RequestAction, RequestDone, RequestMove, RequestRefusal, RequestSlot, RequestStep, RequestTick,
    SlotRef, StackRequestEncoder,
};

/// The slots of the player inventory as one read gives them: 36 entries, `None` for an empty slot.
type Slots = Vec<Option<RequestSlot>>;

/// The slots the client shows for a move it predicts: the two slots and the net id of the stack moved into the second.
type Shown = Option<(u8, u8, i32)>;

/// For each move of a plan: the action, and what the client shows.
type Plan = (Vec<RequestAction>, Vec<Shown>);

/// The one request of `inventory.request` that runs at a time, as a machine with no game access.
///
/// The book decides and the loader does: `tick` gives the acts to do (`RequestStep`) and the requests that ended, and
/// the loader runs them with no lock held, so an answer that comes inside the game's send cannot meet the lock. The
/// loader tells the book what comes in (`on_container_open`, `on_container_close`, `on_responses`) and what it reads
/// (`tick`'s `screen_open` and slot read).
///
/// A request starts with `start` and does nothing until the next `tick`. A hidden-open request sends `Interact`, waits
/// for the server's `ContainerOpen` (which the loader hides from the game), sends every move in one packet at the next
/// tick, and closes the inventory when the answers are in or their time is up; the server's answer to the close is
/// hidden too. An answer is only recorded when it comes; the request is settled by the next `tick`, so the end of a
/// request is never reported inside the call that started it, nor inside the game's send.
///
/// Time is in seconds from any fixed start.
pub struct RequestBook {
    next_id: u64,
    next_wire: i32,
    active: Option<ActiveRequest>,
    // The server's answer to our close is hidden until the time, when it names the container id.
    closing: Option<(f64, i8)>,
    // A late open of an inventory we asked for is still hidden until the time.
    orphan_until: Option<f64>,
    // A late open we hid: its container still needs its close at the next tick.
    owed_close: Option<i8>,
}

impl Default for RequestBook {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestBook {
    /// The most moves of one request.
    pub const MAX_MOVES: usize = 64;
    /// How long the server's open and its answer to our close are waited for.
    pub const ANSWER_WINDOW_SECONDS: f64 = 2.0;
    /// How long the answers to the sent requests are waited for.
    pub const BATCH_TIMEOUT_SECONDS: f64 = 2.0;
    /// How long the slots may differ from what the moves need before the request is stale.
    pub const SETTLE_SECONDS: f64 = 0.5;
    /// The first request id on the wire; each next one is `WIRE_STEP` lower, so all stay odd and negative.
    pub const FIRST_WIRE_ID: i32 = -100_001;
    const WIRE_STEP: i32 = 2;
    const SLOTS: usize = 36;

    pub fn new() -> Self {
        Self {
            next_id: 1,
            next_wire: Self::FIRST_WIRE_ID,
            active: None,
            closing: None,
            orphan_until: None,
            owed_close: None,
        }
    }

    /// Takes a request of the mod `owner`: gives its id. It runs at the next `tick`. `screen_open` is true unless the
    /// HUD is the top screen.
    pub fn start(
        &mut self,
        owner: usize,
        moves: &[RequestMove],
        hidden_open: bool,
        predict: bool,
        screen_open: bool,
    ) -> Result<u64, RequestRefusal> {
        if moves.is_empty() {
            return Err(RequestRefusal::Empty);
        }
        if moves.len() > Self::MAX_MOVES {
            return Err(RequestRefusal::TooMany);
        }
        for request_move in moves {
            Self::check(request_move)?;
        }
        if self.active.is_some() {
            return Err(RequestRefusal::Busy);
        }
        if hidden_open && screen_open {
            return Err(RequestRefusal::ScreenOpen);
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.active = Some(ActiveRequest {
            id,
            owner,
            moves: moves.to_vec(),
            hidden_open,
            predict,
            phase: RequestPhase::Starting,
            wire_ids: Vec::new(),
            answers: Vec::new(),
            predicted: Vec::new(),
            settle_from: None,
            cancel_pending: false,
            detached: false,
        });
        Ok(id)
    }

    /// The server's `ContainerOpen`: true hides it from the game, when it is the open of the inventory a request
    /// asked for.
    pub fn on_container_open(&mut self, now: f64, container_id: i8, container_type: i8) -> bool {
        if container_type != ProtocolValues::INVENTORY_TYPE {
            return false;
        }
        if let Some(active) = self.active.as_mut()
            && let RequestPhase::Opening { until } = active.phase
            && now <= until
        {
            active.phase = RequestPhase::Opened { container_id };
            return true;
        }
        if self.orphan_until.is_some_and(|until| now <= until) {
            self.orphan_until = None;
            self.owed_close = Some(container_id);
            return true;
        }
        false
    }

    /// The server's `ContainerClose`: true hides it from the game, when it is the answer to our close.
    pub fn on_container_close(&mut self, now: f64, container_id: i8) -> bool {
        match self.closing {
            Some((until, closed)) if now <= until && closed == container_id => {
                self.closing = None;
                true
            }
            _ => false,
        }
    }

    /// The answers of an `ItemStackResponse`: the wire request id and whether the server accepted it. An id that is
    /// not one of the sent requests is ignored. The request is settled at the next `tick`.
    pub fn on_responses(&mut self, answers: &[(i32, bool)]) {
        let Some(active) = self.active.as_mut() else { return };
        if !matches!(active.phase, RequestPhase::Sent { .. }) {
            return;
        }
        for (wire_id, accepted) in answers {
            if let Some(index) = active.wire_ids.iter().position(|id| id == wire_id) {
                active.answers[index].get_or_insert(*accepted);
            }
        }
    }

    /// One client tick. `screen_open` is true unless the HUD is the top screen; `read` gives the 36 slots of the
    /// player inventory, or `None` when they cannot be read (it is called only when a request needs them).
    pub fn tick(&mut self, now: f64, screen_open: bool, read: &mut dyn FnMut() -> Option<Slots>) -> RequestTick {
        let mut out = RequestTick::default();
        if self.closing.is_some_and(|(until, _)| now > until) {
            self.closing = None;
        }
        if self.orphan_until.is_some_and(|until| now > until) {
            self.orphan_until = None;
        }
        if let Some(container_id) = self.owed_close.take() {
            self.close(now, container_id, &mut out);
        }
        if let Some(active) = self.active.take() {
            self.active = self.advance(active, now, screen_open, read, &mut out);
        }
        out
    }

    /// A mod is disabled (`owner` is `Some`), or the loader is (`None`: every request): the request ends with no
    /// event. While the game runs, the hidden inventory it opened is closed.
    pub fn disable(&mut self, owner: Option<usize>, now: f64, game_alive: bool) -> Vec<RequestStep> {
        let mut out = RequestTick::default();
        if self.active.as_ref().is_some_and(|active| owner.is_none_or(|owner| active.owner == owner))
            && let Some(active) = self.active.take()
            && game_alive
        {
            match active.phase {
                RequestPhase::Opened { container_id } => self.close(now, container_id, &mut out),
                RequestPhase::Sent { container_id: Some(container_id), .. } => self.close(now, container_id, &mut out),
                RequestPhase::Opening { .. } => self.orphan_until = Some(now + Self::ANSWER_WINDOW_SECONDS),
                _ => {}
            }
        }
        if !game_alive {
            self.forget_windows();
        }
        out.steps
    }

    /// Whether the next `tick` may read the slots: the request sends its moves at that tick. The loader reads them
    /// before it takes the book's lock, so no game call runs under it.
    pub fn wants_slots(&self) -> bool {
        self.active.as_ref().is_some_and(|active| match active.phase {
            RequestPhase::Starting => !active.hidden_open,
            RequestPhase::Opened { .. } => true,
            RequestPhase::Opening { .. } | RequestPhase::Sent { .. } => false,
        })
    }

    /// Whether the book has nothing to do: no request runs and no window for a hidden answer is open. The loader keeps
    /// the packet detours on the game's path only while this is false.
    pub fn is_idle(&self) -> bool {
        self.active.is_none() && self.closing.is_none() && self.orphan_until.is_none() && self.owed_close.is_none()
    }

    /// The world was left: the request ends `CANCELLED` and nothing is sent (the connection is gone). Gives the end to
    /// report; none when no request runs or its end was reported already (a screen opened after the send).
    pub fn world_left(&mut self) -> Vec<RequestDone> {
        let mut out = RequestTick::default();
        if let Some(active) = self.active.take() {
            Self::end(&active, AmmlRequestOutcome::CANCELLED, &mut out);
        }
        self.forget_windows();
        out.done
    }

    /// The loader could not send the packet it was told to (`SendInteract` or `SendRequests`): the request ends
    /// `NOT_SENT` now. Nothing was predicted by a step after the failed one (the loader stops there). A hidden inventory
    /// the server opened is closed.
    pub fn send_failed(&mut self, now: f64) -> RequestTick {
        let mut out = RequestTick::default();
        let Some(active) = self.active.take() else { return out };
        if let RequestPhase::Sent { container_id: Some(container_id), .. } = active.phase {
            self.close(now, container_id, &mut out);
        }
        Self::end(&active, AmmlRequestOutcome::NOT_SENT, &mut out);
        out
    }

    fn forget_windows(&mut self) {
        self.closing = None;
        self.orphan_until = None;
        self.owed_close = None;
    }

    fn check(request_move: &RequestMove) -> Result<(), RequestRefusal> {
        let slots = u8::try_from(Self::SLOTS).unwrap_or(u8::MAX);
        let ok = match *request_move {
            RequestMove::Swap { a, b } => a < slots && b < slots && a != b,
            RequestMove::Place { from, to, count } => from < slots && to < slots && from != to && count > 0,
            RequestMove::Drop { from, count } => from < slots && count > 0,
            RequestMove::Pick => return Err(RequestRefusal::Unsupported),
        };
        if ok { Ok(()) } else { Err(RequestRefusal::BadMove) }
    }

    fn advance(
        &mut self,
        mut active: ActiveRequest,
        now: f64,
        screen_open: bool,
        read: &mut dyn FnMut() -> Option<Slots>,
        out: &mut RequestTick,
    ) -> Option<ActiveRequest> {
        match active.phase {
            RequestPhase::Starting if active.hidden_open => {
                if screen_open {
                    Self::end(&active, AmmlRequestOutcome::CANCELLED, out);
                    return None;
                }
                active.phase = RequestPhase::Opening { until: now + Self::ANSWER_WINDOW_SECONDS };
                out.steps.push(RequestStep::SendInteract);
                Some(active)
            }
            RequestPhase::Starting => self.send(active, None, now, read, out),
            RequestPhase::Opening { until } => {
                active.cancel_pending |= screen_open;
                if now <= until {
                    return Some(active);
                }
                // The open may still come; it is the one we asked for.
                self.orphan_until = Some(now + Self::ANSWER_WINDOW_SECONDS);
                let outcome =
                    if active.cancel_pending { AmmlRequestOutcome::CANCELLED } else { AmmlRequestOutcome::NO_OPEN };
                Self::end(&active, outcome, out);
                None
            }
            RequestPhase::Opened { container_id } => {
                if screen_open || active.cancel_pending {
                    self.close(now, container_id, out);
                    Self::end(&active, AmmlRequestOutcome::CANCELLED, out);
                    return None;
                }
                self.send(active, Some(container_id), now, read, out)
            }
            RequestPhase::Sent { container_id, sent_at } => {
                if active.hidden_open && screen_open && !active.detached {
                    // The server still does the moves; the end is reported now and the answers are still owed.
                    Self::end(&active, AmmlRequestOutcome::CANCELLED, out);
                    active.detached = true;
                }
                let answered = active.answers.iter().all(Option::is_some);
                if answered || now - sent_at >= Self::BATCH_TIMEOUT_SECONDS {
                    self.settle(&active, container_id, now, out);
                    return None;
                }
                Some(active)
            }
        }
    }

    // Makes the requests from the slots now and sends them in one packet; or keeps waiting for the slots to hold what
    // the moves need, and ends stale when they do not in time.
    fn send(
        &mut self,
        mut active: ActiveRequest,
        container_id: Option<i8>,
        now: f64,
        read: &mut dyn FnMut() -> Option<Slots>,
        out: &mut RequestTick,
    ) -> Option<ActiveRequest> {
        let plan = read().filter(|slots| slots.len() == Self::SLOTS).and_then(|slots| Self::plan(&active, slots));
        let Some((actions, predicted)) = plan else {
            let from = *active.settle_from.get_or_insert(now);
            if now - from < Self::SETTLE_SECONDS {
                return Some(active);
            }
            if let Some(container_id) = container_id {
                self.close(now, container_id, out);
            }
            Self::end(&active, AmmlRequestOutcome::STALE, out);
            return None;
        };
        let mut requests = Vec::with_capacity(actions.len());
        for action in actions {
            active.wire_ids.push(self.next_wire);
            requests.push((self.next_wire, action));
            self.next_wire = self.next_wire.wrapping_sub(Self::WIRE_STEP);
        }
        out.steps.push(RequestStep::SendRequests(StackRequestEncoder::batch(&requests)));
        for (a, b, _) in predicted.iter().flatten() {
            out.steps.push(RequestStep::Predict { a: *a, b: *b });
        }
        active.answers = vec![None; active.moves.len()];
        active.predicted = predicted;
        active.phase = RequestPhase::Sent { container_id, sent_at: now };
        Some(active)
    }

    // Ends a request whose answers are all in or whose time is up. All accepted: done. Otherwise every move shown on
    // the client is put back, newest first, and the moves the server did are shown again in order, so the client
    // shows what the server did.
    fn settle(&mut self, active: &ActiveRequest, container_id: Option<i8>, now: f64, out: &mut RequestTick) {
        let refused = active.answers.iter().any(|answer| *answer != Some(true));
        let outcome = if !refused {
            AmmlRequestOutcome::DONE
        } else {
            for (a, b, net_id) in active.predicted.iter().rev().flatten() {
                out.steps.push(RequestStep::Unpredict { a: *a, b: *b, net_id: *net_id });
            }
            for (answer, shown) in active.answers.iter().zip(&active.predicted) {
                if let (Some(true), Some((a, b, _))) = (answer, shown) {
                    out.steps.push(RequestStep::Predict { a: *a, b: *b });
                }
            }
            if active.answers.iter().any(Option::is_none) {
                AmmlRequestOutcome::NO_ANSWER
            } else {
                AmmlRequestOutcome::REFUSED
            }
        };
        if let Some(container_id) = container_id {
            self.close(now, container_id, out);
        }
        Self::end(active, outcome, out);
    }

    // Sends the close; the window for the server's answer opens before the send, as a local server may answer inside it.
    fn close(&mut self, now: f64, container_id: i8, out: &mut RequestTick) {
        self.closing = Some((now + Self::ANSWER_WINDOW_SECONDS, container_id));
        out.steps.push(RequestStep::SendClose(container_id));
    }

    // Reports the end of `active`, unless a cancel reported it already.
    fn end(active: &ActiveRequest, outcome: AmmlRequestOutcome, out: &mut RequestTick) {
        if active.detached {
            return;
        }
        let accepted = active.answers.iter().filter(|answer| **answer == Some(true)).count();
        out.done.push(RequestDone {
            id: active.id,
            outcome,
            accepted: u16::try_from(accepted).unwrap_or(u16::MAX),
            total: u16::try_from(active.moves.len()).unwrap_or(u16::MAX),
        });
    }

    // The action of each move on a model of the slots with the moves before it done, and the moves that can be shown
    // on the client at once; `None` when a move does not fit the slots.
    fn plan(active: &ActiveRequest, slots: Slots) -> Option<Plan> {
        let mut model = slots;
        let mut actions = Vec::with_capacity(active.moves.len());
        let mut predicted = Vec::with_capacity(active.moves.len());
        for request_move in &active.moves {
            let (action, shown) = Self::plan_move(request_move, &mut model)?;
            actions.push(action);
            predicted.push(shown.filter(|_| active.predict));
        }
        Some((actions, predicted))
    }

    fn plan_move(request_move: &RequestMove, model: &mut Slots) -> Option<(RequestAction, Shown)> {
        let slot_ref = |model: &Slots, slot: u8| {
            let net_id = model.get(usize::from(slot)).copied().flatten().map_or(0, |held| held.net_id);
            let (container, slot) = ProtocolValues::player_slot(slot);
            SlotRef { container, slot, net_id }
        };
        match *request_move {
            RequestMove::Swap { a, b } => {
                let first = model.get(usize::from(a)).copied().flatten()?;
                model.get(usize::from(b)).copied().flatten()?;
                let action = RequestAction::Swap { from: slot_ref(model, a), to: slot_ref(model, b) };
                model.swap(usize::from(a), usize::from(b));
                Some((action, Some((a, b, first.net_id))))
            }
            RequestMove::Place { from, to, count } => {
                let source = model.get(usize::from(from)).copied().flatten()?;
                let target = model.get(usize::from(to)).copied()?;
                if count == 0 || count > source.count {
                    return None;
                }
                let (from_ref, to_ref) = (slot_ref(model, from), slot_ref(model, to));
                let (action, moved) = match target {
                    None => {
                        if count > source.max {
                            return None;
                        }
                        (
                            RequestAction::Place { count, from: from_ref, to: to_ref },
                            Some(RequestSlot { count, ..source }),
                        )
                    }
                    Some(held) => {
                        let total = u16::from(held.count) + u16::from(count);
                        if held.key == 0 || held.key != source.key || total > u16::from(held.max) {
                            return None;
                        }
                        (
                            RequestAction::PlaceOnto { count, from: from_ref, to: to_ref },
                            Some(RequestSlot { count: held.count + count, ..held }),
                        )
                    }
                };
                let shown = (target.is_none() && source.count == count).then_some((from, to, source.net_id));
                let left = source.count - count;
                model[usize::from(to)] = moved;
                model[usize::from(from)] = (left > 0).then_some(RequestSlot { count: left, ..source });
                Some((action, shown))
            }
            RequestMove::Drop { from, count } => {
                let source = model.get(usize::from(from)).copied().flatten()?;
                if count == 0 || count > source.count {
                    return None;
                }
                let action = RequestAction::Drop { count, from: slot_ref(model, from) };
                let left = source.count - count;
                model[usize::from(from)] = (left > 0).then_some(RequestSlot { count: left, ..source });
                Some((action, None))
            }
            RequestMove::Pick => None,
        }
    }
}
