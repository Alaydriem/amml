use std::cell::{Cell, RefCell};

use amml_abi::{
    AmmlInputBlocks, AmmlLeaseState, AmmlModifiers, AmmlMouseButton, AmmlMoveIntent, AmmlReleaseReason, AmmlStatus,
    InjectedInputReleased,
};
use amml_runtime::{
    ButtonTable, FrameInjection, InjectedOp, InjectionQueue, KeyTable, KeyVerdict, LeaseBook, LeaseNotice, LeaseTerms,
    NoticeKind, ResourceId,
};

/// The input service of a `FakeGame`, on the loader's own rules: the `LeaseBook` decides which leases
/// are active, the `InjectionQueue` holds input until `FakeGame::frame`, and the `KeyTable` and
/// `ButtonTable` decide which injected downs and ups reach the game. Every lease belongs to the fake's
/// one mod. The fake game's gate and area book are checked by `FakeState` before a call gets here.
pub(crate) struct FakeInput {
    // In the high 32 bits of every lease and hold id, so an id of another fake is unknown here.
    tag: u32,
    next: Cell<u32>,
    keys: KeyTable,
    buttons: ButtonTable,
    book: LeaseBook,
    queue: InjectionQueue,
    leases: RefCell<Vec<ResourceId>>,
    // Per hold: the hold id, its lease and its down.
    holds: RefCell<Vec<(ResourceId, ResourceId, InjectedOp)>>,
    bound: Cell<bool>,
    closed: Cell<bool>,
    injected: RefCell<Vec<String>>,
    notices: RefCell<Vec<(u64, NoticeKind)>>,
    last_move: Cell<Option<AmmlMoveIntent>>,
}

impl FakeInput {
    /// The owner every lease of the fake belongs to.
    const OWNER: usize = 1;
    /// The owner of the leases `acquire_other` takes for other mods.
    const OTHER_OWNER: usize = 2;

    pub(crate) fn new(tag: u32) -> Self {
        Self {
            tag,
            next: Cell::new(1),
            keys: KeyTable::new(),
            buttons: ButtonTable::new(),
            book: LeaseBook::new(),
            queue: InjectionQueue::new(),
            leases: RefCell::new(Vec::new()),
            holds: RefCell::new(Vec::new()),
            bound: Cell::new(false),
            closed: Cell::new(false),
            injected: RefCell::new(Vec::new()),
            notices: RefCell::new(Vec::new()),
            last_move: Cell::new(None),
        }
    }

    fn next_id(&self) -> ResourceId {
        let id = self.next.get();
        self.next.set(id.wrapping_add(1));
        ResourceId((u64::from(self.tag) << 32) | u64::from(id))
    }

    /// Marks every input feed bound, as the loader does at the first real feed of each kind.
    pub(crate) fn bind(&self) {
        self.bound.set(true);
    }

    pub(crate) fn modifiers(&self) -> AmmlModifiers {
        self.keys.modifiers()
    }

    pub(crate) fn is_down(&self, key: u16) -> Option<bool> {
        self.keys.is_down(key)
    }

    /// Takes a lease for the fake's mod. A lease the book refuses, or any lease while the mod is
    /// disabled, gives `REFUSED`.
    pub(crate) fn acquire(&self, module: &str, terms: LeaseTerms) -> Result<u64, AmmlStatus> {
        if self.closed.get() {
            return Err(AmmlStatus::REFUSED);
        }
        let lease = self.next_id();
        let notices = self.book.acquire(lease, Self::OWNER, module, terms).map_err(|_| AmmlStatus::REFUSED)?;
        self.leases.borrow_mut().push(lease);
        self.apply_notices(notices);
        Ok(lease.0)
    }

    /// The active leases of the owners other than the fake's mod: how many, and their blocks.
    pub(crate) fn others(&self) -> (u32, AmmlInputBlocks) {
        self.book.active_of_others(Self::OWNER)
    }

    /// Takes a lease for another mod (`module`), so the fake's mod sees it in `held_by_others`.
    pub(crate) fn acquire_other(&self, module: &str, terms: LeaseTerms) -> bool {
        let lease = self.next_id();
        match self.book.acquire(lease, Self::OTHER_OWNER, module, terms) {
            Ok(notices) => {
                self.apply_notices(notices);
                true
            }
            Err(_) => false,
        }
    }

    pub(crate) fn state(&self, lease: u64) -> AmmlLeaseState {
        self.book.state(ResourceId(lease))
    }

    /// Stores a hold for the down `op` under `lease` and queues the down, as the loader does.
    pub(crate) fn press(&self, lease: u64, op: InjectedOp) -> Result<u64, AmmlStatus> {
        let lease = ResourceId(lease);
        self.check(lease)?;
        let hold = self.next_id();
        self.holds.borrow_mut().push((hold, lease, op));
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
        Ok(hold.0)
    }

    /// Releases a hold; the up is queued when the lease holds that key or button through no other
    /// hold. An unknown or released hold changes nothing.
    pub(crate) fn release_hold(&self, hold: u64) {
        let held = {
            let mut holds = self.holds.borrow_mut();
            let Some(index) = holds.iter().position(|(id, _, _)| id.0 == hold) else {
                return;
            };
            let held = holds.remove(index);
            if holds.iter().any(|(_, lease, op)| *lease == held.1 && *op == held.2) {
                return;
            }
            held
        };
        match held.2 {
            InjectedOp::Key { key, .. } => {
                self.queue.key(held.1, key, false);
            }
            InjectedOp::Button { button, .. } => {
                self.queue.button(held.1, button, false);
            }
            InjectedOp::Wheel { .. } => {}
        }
    }

    pub(crate) fn wheel(&self, lease: u64, notches: f32) -> Result<(), AmmlStatus> {
        let lease = ResourceId(lease);
        self.check(lease)?;
        self.queue.wheel(lease, notches);
        Ok(())
    }

    pub(crate) fn turn(&self, lease: u64, yaw: f32, pitch: f32) -> Result<(), AmmlStatus> {
        let lease = ResourceId(lease);
        self.check(lease)?;
        self.queue.turn(lease, yaw, pitch);
        Ok(())
    }

    pub(crate) fn set_move(&self, lease: u64, intent: Option<AmmlMoveIntent>) -> Result<(), AmmlStatus> {
        let lease = ResourceId(lease);
        self.check(lease)?;
        let notices = self.queue.set_move(lease, Self::OWNER, intent);
        self.apply_notices(notices);
        Ok(())
    }

    /// Releases every hold, the movement and the queued input of `lease`; the lease stays.
    pub(crate) fn release_all(&self, lease: u64) {
        let lease = ResourceId(lease);
        self.drop_holds(lease);
        let _ = self.queue.release(lease);
    }

    /// Ends `lease`: its input is released (`Released(LEASE_ENDED)` when it held any) and the leases
    /// that waited for it are judged again. An unknown lease changes nothing.
    pub(crate) fn end(&self, lease: u64) {
        let lease = ResourceId(lease);
        {
            let mut leases = self.leases.borrow_mut();
            let Some(index) = leases.iter().position(|live| *live == lease) else {
                return;
            };
            leases.remove(index);
        }
        self.drop_holds(lease);
        if self.queue.release(lease).had_input {
            self.notices.borrow_mut().push((lease.0, NoticeKind::Released(AmmlReleaseReason::LEASE_ENDED)));
        }
        let notices = self.book.end(lease);
        self.apply_notices(notices);
    }

    pub(crate) fn set_ranking(&self, ids: Vec<String>) -> Result<(), AmmlStatus> {
        self.book.set_ranking(Self::OWNER, ids).map_err(|_| AmmlStatus::REFUSED)
    }

    pub(crate) fn set_coexist(&self, pairs: Vec<(String, String)>) -> Result<(), AmmlStatus> {
        let notices = self.book.set_coexist(Self::OWNER, pairs).map_err(|_| AmmlStatus::REFUSED)?;
        self.apply_notices(notices);
        Ok(())
    }

    /// Ends every lease for a disable, with no notice to the mod, and refuses new leases until
    /// `reopen`.
    pub(crate) fn end_owner(&self) {
        self.closed.set(true);
        let ended = std::mem::take(&mut *self.leases.borrow_mut());
        for lease in ended {
            self.drop_holds(lease);
            let _ = self.queue.release(lease);
        }
        let _ = self.book.end_owner(Self::OWNER);
        self.notices.borrow_mut().clear();
    }

    pub(crate) fn reopen(&self) {
        self.closed.set(false);
    }

    /// The lease notices since the last frame, in order.
    pub(crate) fn take_notices(&self) -> Vec<(u64, NoticeKind)> {
        std::mem::take(&mut *self.notices.borrow_mut())
    }

    pub(crate) fn take_frame(&self) -> FrameInjection {
        self.queue.take_frame()
    }

    /// Records one real key feed before the listeners (`KeyTable::real`). A real up that ended
    /// injected holds of the key ends those holds, and each lease that held it gets `Released(RESET)`
    /// naming the key, as in the loader.
    pub(crate) fn real_key(&self, key: u16, down: bool) -> KeyVerdict {
        let verdict = self.keys.real(key, down);
        let op = InjectedOp::Key { key, down: true };
        for &lease in &verdict.ended {
            let dropped = {
                let mut holds = self.holds.borrow_mut();
                let before = holds.len();
                holds.retain(|(_, holder, held)| !(*holder == lease && *held == op));
                holds.len() != before
            };
            self.queue.forget(lease, op);
            if dropped {
                let kind = NoticeKind::ReleasedOne {
                    reason: AmmlReleaseReason::RESET,
                    input: InjectedInputReleased::KEY,
                    code: key,
                };
                self.notices.borrow_mut().push((lease.0, kind));
            }
        }
        verdict
    }

    /// Whether an active lease blocks the real `key`: it blocks `KEYS` and no passthrough names the key.
    pub(crate) fn blocks_key(&self, key: u16) -> bool {
        self.book.blocks_now().0 & AmmlInputBlocks::KEYS.0 != 0 && !self.book.passes(key)
    }

    /// Records the listeners' answer to a real key; gives whether the game gets it (`KeyTable::finish`).
    pub(crate) fn finish_real_key(&self, key: u16, down: bool, cancelled: bool) -> bool {
        self.keys.finish(key, down, cancelled)
    }

    /// Whether an injected key reaches the game (`KeyTable::injected`).
    pub(crate) fn key_reaches_game(&self, lease: ResourceId, key: u16, down: bool) -> bool {
        self.keys.injected(lease, key, down)
    }

    /// Whether an injected button reaches the game (`ButtonTable::injected`).
    pub(crate) fn button_reaches_game(&self, lease: ResourceId, button: AmmlMouseButton, down: bool) -> bool {
        self.buttons.injected(lease, button, down)
    }

    /// Whether the move intent differs from the one recorded last; records it.
    pub(crate) fn move_changed(&self, intent: Option<AmmlMoveIntent>) -> bool {
        let changed = self.last_move.get() != intent;
        self.last_move.set(intent);
        changed
    }

    pub(crate) fn record(&self, line: String) {
        self.injected.borrow_mut().push(line);
    }

    pub(crate) fn injected(&self) -> Vec<String> {
        self.injected.borrow().clone()
    }

    // A known, active lease while the feeds are bound; `REFUSED` or `NOT_READY` otherwise.
    fn check(&self, lease: ResourceId) -> Result<(), AmmlStatus> {
        if !self.leases.borrow().contains(&lease) || self.book.state(lease) != AmmlLeaseState::ACTIVE {
            return Err(AmmlStatus::REFUSED);
        }
        if !self.bound.get() {
            return Err(AmmlStatus::NOT_READY);
        }
        Ok(())
    }

    // A lease that is suspended or lost has its input released first; its owner also gets
    // `Released(SUSPENDED)` when it held any, as in the loader.
    fn apply_notices(&self, notices: Vec<LeaseNotice>) {
        for LeaseNotice { lease, kind, .. } in notices {
            match kind {
                NoticeKind::Suspended => {
                    self.notices.borrow_mut().push((lease.0, NoticeKind::Suspended));
                    self.drop_holds(lease);
                    if self.queue.release(lease).had_input {
                        let released = NoticeKind::Released(AmmlReleaseReason::SUSPENDED);
                        self.notices.borrow_mut().push((lease.0, released));
                    }
                }
                NoticeKind::Lost(_) => {
                    self.drop_holds(lease);
                    let _ = self.queue.release(lease);
                    self.notices.borrow_mut().push((lease.0, kind));
                }
                NoticeKind::Resumed | NoticeKind::Released(_) | NoticeKind::ReleasedOne { .. } => {
                    self.notices.borrow_mut().push((lease.0, kind));
                }
            }
        }
    }

    fn drop_holds(&self, lease: ResourceId) {
        self.holds.borrow_mut().retain(|(_, holder, _)| *holder != lease);
    }
}
