use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use amml_abi::{
    AmmlLeaseState, AmmlMouseAction, AmmlMouseButton, AmmlMoveIntent, AmmlPhase, AmmlReleaseReason, AmmlStatus,
    InjectedInputReleased, KeyInput, LookDelta, MouseInput,
};
use amml_runtime::{
    ButtonTable, InjectedOp, InjectionQueue, KeyTable, KeyVerdict, LeaseBook, LeaseNotice, LeaseRefusal, LeaseTerms,
    NoticeKind, ResourceId, ResourceKind, ResourceStore, WorldEpoch,
};

use crate::{AreaBook, EventHub, InputHold, InputLedger, KeyFeed, MouseFeed, OwnerEvent, OwnerEvents, TurnFeed};

/// The loader's input service: the key table, the lease book, the injection queue and the feeds the
/// build's input adapters bind.
///
/// Leases and holds are resources of their mod in the loader's store, bound to no world (epoch 0):
/// a world leave releases what they hold but keeps them. Every change to leases, holds and queued
/// input runs under one lock, so a lease that is suspended or ended can never queue input after its
/// release. The lease notices go to `OwnerEvents` and reach their mods at the next frame.
pub struct InputService {
    keys: KeyTable,
    buttons: ButtonTable,
    book: LeaseBook,
    queue: InjectionQueue,
    ledger: Mutex<InputLedger>,
    key_feed: Mutex<Option<Arc<dyn KeyFeed>>>,
    mouse_feed: Mutex<Option<Arc<dyn MouseFeed>>>,
    turn_feed: Mutex<Option<Arc<dyn TurnFeed>>>,
    move_bound: AtomicBool,
    hub: Arc<EventHub>,
    areas: Arc<AreaBook>,
    resources: Arc<ResourceStore<usize>>,
    owner_events: Arc<OwnerEvents>,
}

impl InputService {
    /// The resource kind of an input lease.
    pub const LEASE: ResourceKind = ResourceKind(1);
    /// The resource kind of a key hold.
    pub const KEY_HOLD: ResourceKind = ResourceKind(2);
    /// The resource kind of a mouse button hold.
    pub const BUTTON_HOLD: ResourceKind = ResourceKind(3);
    /// The most wheel notches one `wheel` call sends; the rest of a larger call is dropped.
    pub const MAX_WHEEL_NOTCHES: u32 = 100;

    const KEY_INJECT: &'static str = "input.key_inject";
    const MOUSE_INJECT: &'static str = "input.mouse_inject";
    const TURN: &'static str = "input.turn";
    const MOVE: &'static str = "input.move";
    const AREA_VERSION: u32 = 1;
    const NO_WORLD: WorldEpoch = WorldEpoch(0);

    pub fn new(
        hub: Arc<EventHub>,
        areas: Arc<AreaBook>,
        resources: Arc<ResourceStore<usize>>,
        owner_events: Arc<OwnerEvents>,
    ) -> Self {
        Self {
            keys: KeyTable::new(),
            buttons: ButtonTable::new(),
            book: LeaseBook::new(),
            queue: InjectionQueue::new(),
            ledger: Mutex::new(InputLedger::default()),
            key_feed: Mutex::new(None),
            mouse_feed: Mutex::new(None),
            turn_feed: Mutex::new(None),
            move_bound: AtomicBool::new(false),
            hub,
            areas,
            resources,
            owner_events,
        }
    }

    /// The keys the real feed holds down.
    pub fn keys(&self) -> &KeyTable {
        &self.keys
    }

    /// The mouse buttons the real feed holds down, and the injected holders of each.
    pub fn buttons(&self) -> &ButtonTable {
        &self.buttons
    }

    /// The leases, for the adapters that enforce their blocks on real input.
    pub fn book(&self) -> &LeaseBook {
        &self.book
    }

    pub fn bind_keys(&self, feed: Arc<dyn KeyFeed>) {
        curia::info!("input feed bound", { area: Self::KEY_INJECT, shape: feed.shape() });
        *Self::lock_slot(&self.key_feed) = Some(feed);
    }

    pub fn bind_mouse(&self, feed: Arc<dyn MouseFeed>) {
        curia::info!("input feed bound", { area: Self::MOUSE_INJECT, shape: feed.shape() });
        *Self::lock_slot(&self.mouse_feed) = Some(feed);
    }

    pub fn bind_turn(&self, feed: Arc<dyn TurnFeed>) {
        curia::info!("input feed bound", { area: Self::TURN });
        *Self::lock_slot(&self.turn_feed) = Some(feed);
    }

    /// Marks the move point bound: its detour writes the move intent.
    pub fn bind_move(&self) {
        curia::info!("input feed bound", { area: Self::MOVE });
        self.move_bound.store(true, Ordering::Release);
    }

    /// Takes a lease for the mod at `owner` (`owner_id` is its mod id, for the ranking). A lease
    /// the book refuses, or a mod whose leases `end_owner` ended, gives `REFUSED`.
    pub fn acquire(&self, owner: usize, owner_id: &str, terms: LeaseTerms) -> Result<ResourceId, AmmlStatus> {
        let mut ledger = self.lock();
        if ledger.closed.contains(&owner) {
            curia::info!("an input lease was refused: the mod is disabled", { mod_name: owner_id });
            return Err(AmmlStatus::REFUSED);
        }
        let lease = self.resources.insert(Self::LEASE, owner, Self::NO_WORLD, 0).map_err(|_| AmmlStatus::FAILED)?;
        match self.book.acquire(lease, owner, owner_id, terms) {
            Ok(notices) => {
                ledger.leases.push((lease, owner));
                self.apply_notices(&mut ledger, notices);
                Ok(lease)
            }
            Err(refusal) => {
                let _ = self.resources.remove(lease);
                curia::info!("an input lease was refused", { mod_name: owner_id, reason: refusal.to_string() });
                Err(AmmlStatus::REFUSED)
            }
        }
    }

    /// The state of `lease`; `LOST` for an unknown or ended lease.
    pub fn lease_state(&self, lease: ResourceId) -> AmmlLeaseState {
        self.book.state(lease)
    }

    /// Presses `key` under `lease` and gives a new hold. The key stays down until every hold of it is
    /// released, from this lease and from any other.
    pub fn key_down(&self, lease: ResourceId, key: u16) -> Result<ResourceId, AmmlStatus> {
        if key == 0 || key >= 0x100 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let ready = || Self::feed_ready(&self.key_feed, |feed| feed.ready());
        self.press(lease, Self::KEY_INJECT, ready, InjectedOp::Key { key, down: true }, Self::KEY_HOLD)
    }

    /// Presses `button` under `lease` and gives a new hold, under the same rule as `key_down`.
    pub fn button_down(&self, lease: ResourceId, button: AmmlMouseButton) -> Result<ResourceId, AmmlStatus> {
        if button.0 > AmmlMouseButton::X2.0 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let ready = || Self::feed_ready(&self.mouse_feed, |feed| feed.ready());
        self.press(lease, Self::MOUSE_INJECT, ready, InjectedOp::Button { button, down: true }, Self::BUTTON_HOLD)
    }

    /// Releases `hold` at the next frame. The up is queued only when the lease holds that key or
    /// button through no other hold. A hold that is already released, or unknown, gives `Ok`.
    pub fn release_hold(&self, hold: ResourceId) -> Result<(), AmmlStatus> {
        let mut ledger = self.lock();
        let Some(index) = ledger.holds.iter().position(|held| held.hold == hold) else {
            return Ok(());
        };
        let held = ledger.holds.remove(index);
        let _ = self.resources.remove(hold);
        if !ledger.holds.iter().any(|other| other.lease == held.lease && other.op == held.op) {
            self.queue_up(&held);
        }
        Ok(())
    }

    /// Scrolls `notches` under `lease`; whole notches reach the game, the fraction waits.
    pub fn wheel(&self, lease: ResourceId, notches: f32) -> Result<(), AmmlStatus> {
        if !notches.is_finite() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let ledger = self.lock();
        let ready = || Self::feed_ready(&self.mouse_feed, |feed| feed.ready());
        self.check(&ledger, lease, Self::MOUSE_INJECT, ready)?;
        self.queue.wheel(lease, notches);
        Ok(())
    }

    /// Adds a turn under `lease` to the frame's sum.
    pub fn turn(&self, lease: ResourceId, yaw: f32, pitch: f32) -> Result<(), AmmlStatus> {
        if !yaw.is_finite() || !pitch.is_finite() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let ledger = self.lock();
        let ready = || Self::feed_ready(&self.turn_feed, |_| Ok(()));
        self.check(&ledger, lease, Self::TURN, ready)?;
        self.queue.turn(lease, yaw, pitch);
        Ok(())
    }

    /// Sets or clears the move intent of `lease`. A new intent replaces the intent of another lease,
    /// whose owner gets `Released(REPLACED)`.
    pub fn set_move(&self, lease: ResourceId, intent: Option<AmmlMoveIntent>) -> Result<(), AmmlStatus> {
        if intent.is_some_and(|intent| !intent.forward.is_finite() || !intent.left.is_finite()) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let mut ledger = self.lock();
        let ready = || if self.move_bound.load(Ordering::Acquire) { Ok(()) } else { Err(AmmlStatus::NOT_READY) };
        let owner = self.check(&ledger, lease, Self::MOVE, ready)?;
        let notices = self.queue.set_move(lease, owner, intent);
        self.apply_notices(&mut ledger, notices);
        Ok(())
    }

    /// The move intent the move point writes, of whichever lease set it last; `None` when no lease has
    /// one. Takes only the queue lock, never the input lock.
    pub fn move_intent(&self) -> Option<AmmlMoveIntent> {
        self.queue.move_intent()
    }

    /// Releases every hold, the move intent and the queued input of `lease`. The lease stays.
    pub fn release_all(&self, lease: ResourceId) {
        let mut ledger = self.lock();
        self.drop_holds(&mut ledger, lease);
        let _ = self.queue.release(lease);
    }

    /// Ends `lease`: its input is released (`Released(LEASE_ENDED)` to its owner when it held any)
    /// and the leases that waited for it are judged again. An unknown or ended lease gives `Ok`.
    pub fn end(&self, lease: ResourceId) -> Result<(), AmmlStatus> {
        let mut ledger = self.lock();
        let Some(index) = ledger.leases.iter().position(|(live, _)| *live == lease) else {
            return Ok(());
        };
        let (_, owner) = ledger.leases.remove(index);
        self.drop_holds(&mut ledger, lease);
        if self.queue.release(lease).had_input {
            self.owner_events
                .push(owner, OwnerEvent::Released { lease: lease.0, reason: AmmlReleaseReason::LEASE_ENDED });
        }
        let _ = self.resources.remove(lease);
        ledger.retiring.push((lease, owner));
        let notices = self.book.end(lease);
        self.apply_notices(&mut ledger, notices);
        Ok(())
    }

    /// Sets the lease ranking for the mod at `owner`; another mod owning it gives `REFUSED`.
    pub fn set_ranking(&self, owner: usize, ids: Vec<String>) -> Result<(), AmmlStatus> {
        self.book.set_ranking(owner, ids).map_err(|refusal: LeaseRefusal| {
            curia::info!("a lease ranking was refused", { owner: owner, reason: refusal.to_string() });
            AmmlStatus::REFUSED
        })
    }

    /// Sets the coexist pairs for the mod at `owner`; another mod owning the lease policy gives
    /// `REFUSED`. The leases the change suspends or resumes get their notices.
    pub fn set_coexist(&self, owner: usize, pairs: Vec<(String, String)>) -> Result<(), AmmlStatus> {
        let mut ledger = self.lock();
        match self.book.set_coexist(owner, pairs) {
            Ok(notices) => {
                self.apply_notices(&mut ledger, notices);
                Ok(())
            }
            Err(refusal) => {
                curia::info!("coexist pairs were refused", { owner: owner, reason: refusal.to_string() });
                Err(AmmlStatus::REFUSED)
            }
        }
    }

    /// Records one real key feed in the key table. A real up that ended injected holds of the key in
    /// the game ends those holds here too, and each owner gets `Released(RESET)` naming the key, so a
    /// new down presses it again.
    pub fn real_key(&self, key: u16, down: bool) -> KeyVerdict {
        let verdict = self.keys.real(key, down);
        self.end_in_game(&verdict.ended, InjectedOp::Key { key, down: true });
        verdict
    }

    /// Records one real button feed, under the rule of `real_key`.
    pub fn real_button(&self, button: AmmlMouseButton, down: bool) {
        let ended = self.buttons.real(button, down);
        self.end_in_game(&ended, InjectedOp::Button { button, down: true });
    }

    // Forgets the holds of `op` (a down) of each lease in `ended`, whose hold a real up ended in the game,
    // and tells each owner that held it.
    fn end_in_game(&self, ended: &[ResourceId], op: InjectedOp) {
        if ended.is_empty() {
            return;
        }
        let (input, code) = match op {
            InjectedOp::Key { key, .. } => (InjectedInputReleased::KEY, key),
            InjectedOp::Button { button, .. } => (InjectedInputReleased::BUTTON, u16::from(button.0)),
            InjectedOp::Wheel { .. } => return,
        };
        let mut ledger = self.lock();
        for &lease in ended {
            let before = ledger.holds.len();
            let resources = &self.resources;
            ledger.holds.retain(|held| {
                if held.lease != lease || held.op != op {
                    return true;
                }
                let _ = resources.remove(held.hold);
                false
            });
            self.queue.forget(lease, op);
            if ledger.holds.len() == before {
                continue;
            }
            if !ledger.reset.contains(&lease) {
                ledger.reset.push(lease);
            }
            if let Some(owner) = Self::owner_in(&ledger, lease) {
                let reason = AmmlReleaseReason::RESET;
                self.owner_events.push(owner, OwnerEvent::ReleasedOne { lease: lease.0, reason, input, code });
            }
        }
    }

    /// Ends every lease of the mod at `owner`, for a disable or unload, and refuses its new leases
    /// until `reopen_owner`. Its input is released with no notice to it: its listeners are gone. The
    /// other mods get their `Resumed` notices.
    pub fn end_owner(&self, owner: usize) {
        let mut ledger = self.lock();
        if !ledger.closed.contains(&owner) {
            ledger.closed.push(owner);
        }
        let (ended, kept): (Vec<_>, Vec<_>) = ledger.leases.iter().copied().partition(|(_, holder)| *holder == owner);
        ledger.leases = kept;
        for (lease, _) in &ended {
            self.drop_holds(&mut ledger, *lease);
            let _ = self.queue.release(*lease);
            let _ = self.resources.remove(*lease);
        }
        ledger.retiring.extend(ended);
        let notices = self.book.end_owner(owner);
        self.apply_notices(&mut ledger, notices);
    }

    /// Accepts new leases of the mod at `owner` again, for its next enable.
    pub fn reopen_owner(&self, owner: usize) {
        self.lock().closed.retain(|closed| *closed != owner);
    }

    /// Releases the injected input of every lease, for a world leave or a focus loss. Each owner
    /// whose lease held a key, a button or the move intent gets `Released(reason)`; so does the owner
    /// of a lease whose holds a real up ended since the last frame, because the game's focus path
    /// feeds its key reset before the loader sees the loss. The leases stay.
    pub fn release_everything(&self, reason: AmmlReleaseReason) {
        let mut ledger = self.lock();
        let released = self.queue.release_everything();
        for held in std::mem::take(&mut ledger.holds) {
            let _ = self.resources.remove(held.hold);
        }
        let mut told = std::mem::take(&mut ledger.reset);
        for (lease, outcome) in released {
            if outcome.had_input && !told.contains(&lease) {
                told.push(lease);
            }
        }
        for lease in told {
            if let Some(owner) = Self::owner_in(&ledger, lease) {
                self.owner_events.push(owner, OwnerEvent::Released { lease: lease.0, reason });
            }
        }
    }

    /// Raises a real up for every key the bound key feed holds down, for a focus loss that reaches the
    /// game. Nothing is sent to the game: it resets its own keys. The game's reset path, when it runs
    /// through a loader hook later, then finds no key down and raises nothing.
    pub fn release_real_keys(&self) {
        let feed = Self::lock_slot(&self.key_feed).clone();
        if let Some(feed) = feed {
            feed.release_real();
        }
    }

    /// Applies the input queued since the last frame. Game thread, at `FrameUpdate` Before.
    ///
    /// Each key goes through `KeyTable::injected` (only the first holder's down and the last holder's
    /// up reach the game, and neither while the real key is down), then `KeyInput` is raised as
    /// injected and the key is fed; buttons go through `ButtonTable::injected` the same way and raise
    /// `MouseInput`; each whole wheel notch raises one `MouseInput` of one notch and is fed once. The
    /// summed turn raises `LookDelta` with no single injector and is fed with the listeners' values.
    /// Input whose feed is not bound is dropped.
    pub fn apply_frame(&self, hub: &EventHub) {
        let retired = {
            let mut ledger = self.lock();
            ledger.reset.clear();
            std::mem::take(&mut ledger.retiring)
        };
        let frame = self.queue.take_frame();
        let owners: Vec<(ResourceId, usize)> = {
            let ledger = self.lock();
            ledger.leases.iter().chain(&ledger.retiring).chain(&retired).copied().collect()
        };
        let context = hub.loader_context();
        let key_feed = Self::lock_slot(&self.key_feed).clone();
        let mouse_feed = Self::lock_slot(&self.mouse_feed).clone();
        for (lease, op) in frame.ops {
            let injector = owners
                .iter()
                .find(|(owner_lease, _)| *owner_lease == lease)
                .map_or(EventHub::NO_INJECTOR, |(_, owner)| *owner);
            match op {
                InjectedOp::Key { key, down } => {
                    let Some(feed) = &key_feed else {
                        continue;
                    };
                    if !self.keys.injected(lease, key, down) {
                        continue;
                    }
                    let mut event = KeyInput {
                        header: EventHub::header::<KeyInput>(AmmlPhase::BEFORE),
                        key,
                        down: u8::from(down),
                        repeat: 0,
                        modifiers: self.keys.modifiers(),
                    };
                    hub.raise_injected(&mut event, &context, injector);
                    feed.send(key, down);
                }
                InjectedOp::Button { button, down } => {
                    let Some(feed) = &mouse_feed else {
                        continue;
                    };
                    if !self.buttons.injected(lease, button, down) {
                        continue;
                    }
                    let mut event = Self::mouse_event(AmmlMouseAction::BUTTON, button, down, 0.0);
                    hub.raise_injected(&mut event, &context, injector);
                    feed.send_button(button, down);
                }
                InjectedOp::Wheel { notches } => {
                    let Some(feed) = &mouse_feed else {
                        continue;
                    };
                    let notch = if notches < 0.0 { -1.0 } else { 1.0 };
                    let whole = notches.abs().trunc();
                    if whole > Self::MAX_WHEEL_NOTCHES as f32 {
                        curia::debug!("an injected wheel was cut", { notches: notches });
                    }
                    for _ in 0..(whole as u32).min(Self::MAX_WHEEL_NOTCHES) {
                        let mut event = Self::mouse_event(AmmlMouseAction::WHEEL, AmmlMouseButton::LEFT, false, notch);
                        hub.raise_injected(&mut event, &context, injector);
                        feed.send_wheel(notch);
                    }
                }
            }
        }
        let turn_feed = Self::lock_slot(&self.turn_feed).clone();
        if let Some((yaw, pitch)) = frame.turn
            && let Some(feed) = turn_feed
        {
            let mut event = LookDelta { header: EventHub::header::<LookDelta>(AmmlPhase::BEFORE), yaw, pitch };
            hub.raise_injected(&mut event, &context, EventHub::NO_INJECTOR);
            if !event.yaw.is_finite() || !event.pitch.is_finite() || !feed.send_turn(event.yaw, event.pitch) {
                curia::debug!("an injected turn was dropped", { yaw: event.yaw, pitch: event.pitch });
            }
        }
    }

    fn mouse_event(action: AmmlMouseAction, button: AmmlMouseButton, down: bool, notches: f32) -> MouseInput {
        MouseInput {
            header: EventHub::header::<MouseInput>(AmmlPhase::BEFORE),
            action,
            button,
            down: u8::from(down),
            _reserved: 0,
            x: 0,
            y: 0,
            dx: 0,
            dy: 0,
            notches,
        }
    }

    // Stores a new hold for `op` (a down) under `lease` and queues the down. The queue drops a down of
    // a key or button the lease already holds; when a real up has ended that hold in the game (the
    // table no longer records the lease as a holder), the down is queued again. A down that is still
    // waiting for its frame is queued twice; the table drops the second.
    fn press(
        &self,
        lease: ResourceId,
        area: &str,
        ready: impl FnOnce() -> Result<(), AmmlStatus>,
        op: InjectedOp,
        kind: ResourceKind,
    ) -> Result<ResourceId, AmmlStatus> {
        let mut ledger = self.lock();
        let owner = self.check(&ledger, lease, area, ready)?;
        let value = usize::try_from(lease.0).map_err(|_| AmmlStatus::FAILED)?;
        let hold = self.resources.insert(kind, owner, Self::NO_WORLD, value).map_err(|_| AmmlStatus::FAILED)?;
        ledger.holds.push(InputHold { hold, lease, op });
        let ended_in_game = match op {
            InjectedOp::Key { key, .. } => !self.queue.key(lease, key, true) && !self.keys.holds(lease, key),
            InjectedOp::Button { button, .. } => {
                !self.queue.button(lease, button, true) && !self.buttons.holds(lease, button)
            }
            InjectedOp::Wheel { .. } => false,
        };
        if ended_in_game {
            self.queue.press_again(lease, op);
        }
        Ok(hold)
    }

    // The owner of `lease` when it may inject into `area` now: the game runs, the build offers the
    // area, the lease is live and active, and the area's feed is ready.
    fn check(
        &self,
        ledger: &InputLedger,
        lease: ResourceId,
        area: &str,
        ready: impl FnOnce() -> Result<(), AmmlStatus>,
    ) -> Result<usize, AmmlStatus> {
        self.hub.gate().require_open()?;
        if !self.areas.is_available(area, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let owner = ledger
            .leases
            .iter()
            .find(|(live, _)| *live == lease)
            .map(|(_, owner)| *owner)
            .ok_or(AmmlStatus::REFUSED)?;
        if self.book.state(lease) != AmmlLeaseState::ACTIVE {
            return Err(AmmlStatus::REFUSED);
        }
        ready()?;
        Ok(owner)
    }

    // `NOT_READY` while no feed is bound in `slot`, else what `ready` says of the feed.
    fn feed_ready<F: ?Sized>(
        slot: &Mutex<Option<Arc<F>>>,
        ready: impl FnOnce(&F) -> Result<(), AmmlStatus>,
    ) -> Result<(), AmmlStatus> {
        let feed = Self::lock_slot(slot).clone().ok_or(AmmlStatus::NOT_READY)?;
        ready(&feed)
    }

    // Raises lease notices to their owners. A lease that is suspended or lost has its input released
    // first; its owner also gets `Released(SUSPENDED)` when it held any.
    fn apply_notices(&self, ledger: &mut InputLedger, notices: Vec<LeaseNotice>) {
        for LeaseNotice { owner, lease, kind } in notices {
            match kind {
                NoticeKind::Suspended => {
                    self.owner_events.push(owner, OwnerEvent::Suspended { lease: lease.0 });
                    self.drop_holds(ledger, lease);
                    if self.queue.release(lease).had_input {
                        let reason = AmmlReleaseReason::SUSPENDED;
                        self.owner_events.push(owner, OwnerEvent::Released { lease: lease.0, reason });
                    }
                }
                NoticeKind::Lost(reason) => {
                    self.drop_holds(ledger, lease);
                    let _ = self.queue.release(lease);
                    self.owner_events.push(owner, OwnerEvent::Lost { lease: lease.0, reason });
                }
                NoticeKind::Resumed => self.owner_events.push(owner, OwnerEvent::Resumed { lease: lease.0 }),
                NoticeKind::Released(reason) => {
                    self.owner_events.push(owner, OwnerEvent::Released { lease: lease.0, reason });
                }
                NoticeKind::ReleasedOne { reason, input, code } => {
                    self.owner_events.push(owner, OwnerEvent::ReleasedOne { lease: lease.0, reason, input, code });
                }
            }
        }
    }

    // Forgets the holds of `lease`; the caller releases the lease's queued input.
    fn drop_holds(&self, ledger: &mut InputLedger, lease: ResourceId) {
        let resources = &self.resources;
        ledger.holds.retain(|held| {
            if held.lease != lease {
                return true;
            }
            let _ = resources.remove(held.hold);
            false
        });
    }

    fn queue_up(&self, held: &InputHold) {
        match held.op {
            InjectedOp::Key { key, .. } => {
                self.queue.key(held.lease, key, false);
            }
            InjectedOp::Button { button, .. } => {
                self.queue.button(held.lease, button, false);
            }
            InjectedOp::Wheel { .. } => {}
        }
    }

    fn owner_in(ledger: &InputLedger, lease: ResourceId) -> Option<usize> {
        ledger.leases.iter().chain(&ledger.retiring).find(|(live, _)| *live == lease).map(|(_, owner)| *owner)
    }

    // The ledger holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, InputLedger> {
        self.ledger.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_slot<T>(slot: &Mutex<T>) -> MutexGuard<'_, T> {
        slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
