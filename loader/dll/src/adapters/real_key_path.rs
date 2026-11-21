use std::sync::atomic::{AtomicU8, Ordering};

use amml_abi::{AmmlEventContext, AmmlInputBlocks, AmmlPhase, AmmlStatus, AmmlThread, KeyInput};

use crate::{EventHub, LoaderState};

/// What both key adapters do with one real key the game feeds: the key table, the `KeyInput` event, the
/// blocks of the active leases, and the thread rule of key injection.
///
/// The thread rule: the first real key down that runs after the game thread is known decides whether
/// the key feed is the game thread. Ups do not decide, because the focus and reset paths of the game
/// send an up for every key, possibly from another thread. When the key down is not on the game thread,
/// key injection is not available for the rest of the process and the `input.key_inject` area is
/// withdrawn.
pub struct RealKeyPath {
    state: &'static LoaderState,
    thread: AtomicU8,
}

impl RealKeyPath {
    const KEY_INJECT: &'static str = "input.key_inject";
    const UNDECIDED: u8 = 0;
    const GAME_THREAD: u8 = 1;
    const OTHER_THREAD: u8 = 2;

    pub fn new(state: &'static LoaderState) -> Self {
        Self { state, thread: AtomicU8::new(Self::UNDECIDED) }
    }

    /// Runs one real key through the loader. Gives whether the game gets it.
    ///
    /// An up of a key the loader did not see down (a reset loop, or the up after an injected hold)
    /// reaches the game and raises no event. Any other key raises `KeyInput`; a key that an active lease
    /// blocks (`KEYS`, and not in every passthrough) is raised already cancelled. The key table then
    /// decides from the cancel whether the game gets the key.
    pub fn feed(&self, key: u16, down: bool) -> bool {
        let hub = self.state.events();
        let on_game_thread = hub.gate().on_game_thread();
        if down {
            self.decide(on_game_thread);
        }
        let input = self.state.input();
        let verdict = input.real_key(key, down);
        if !verdict.raise {
            return true;
        }
        let book = input.book();
        let blocked = book.blocks_now().0 & AmmlInputBlocks::KEYS.0 != 0 && !book.passes(key);
        let mut event = KeyInput {
            header: EventHub::header::<KeyInput>(AmmlPhase::BEFORE),
            key,
            down: u8::from(down),
            repeat: u8::from(verdict.repeat),
            modifiers: input.keys().modifiers(),
        };
        event.header.cancelled = u8::from(blocked);
        let thread = if on_game_thread { AmmlThread::GAME } else { AmmlThread::INPUT };
        let context = AmmlEventContext { thread, ..hub.loader_context() };
        hub.raise(&mut event, &context);
        input.keys().finish(key, down, event.header.cancelled != 0)
    }

    /// Runs a real up through `feed` for every key the key table holds down, as the game's own key reset
    /// does on 70.x: each raises `KeyInput` and ends the injected holds of its key. What `feed` says the
    /// game gets is not used: the caller resets the game's keys itself.
    pub fn release_held(&self) {
        for key in self.state.input().keys().real_down_keys() {
            self.feed(key, false);
        }
    }

    /// `NOT_AVAILABLE` when a real key down ran off the game thread, `Ok` otherwise.
    pub fn thread_allows(&self) -> Result<(), AmmlStatus> {
        if self.thread.load(Ordering::Acquire) == Self::OTHER_THREAD { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(()) }
    }

    /// The game window's procedure runs on the game thread, and the build's key feed is reached only from that
    /// procedure (the `key_processor` shape): the key feed is the game thread, decided now, before any real key. A
    /// later real key down off the game thread still withdraws key injection (see `feed`).
    pub fn decide_from_window(&self) {
        if !self.state.events().gate().is_open() {
            return;
        }
        if self.thread.compare_exchange(Self::UNDECIDED, Self::GAME_THREAD, Ordering::AcqRel, Ordering::Acquire).is_ok()
        {
            curia::info!("key feed thread is the game thread", { from: "window" });
        }
    }

    /// Whether the caller runs on the game thread.
    pub fn on_game_thread(&self) -> bool {
        self.state.events().gate().on_game_thread()
    }

    /// Whether a real key down has decided the thread rule.
    pub fn decided(&self) -> bool {
        self.thread.load(Ordering::Acquire) != Self::UNDECIDED
    }

    // Decides the thread rule at the first key down once the game thread is known (the gate opens at
    // the first frame); before that, nothing is decided.
    // A real key down off the game thread after the rule said "game thread" (decided from the window) withdraws
    // key injection as well.
    fn decide(&self, on_game_thread: bool) {
        if !self.state.events().gate().is_open() {
            return;
        }
        if !on_game_thread
            && self
                .thread
                .compare_exchange(Self::GAME_THREAD, Self::OTHER_THREAD, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        {
            curia::warn!("key injection not available: the key feed is not the game thread");
            self.state.areas().withdraw(Self::KEY_INJECT);
            return;
        }
        if self.decided() {
            return;
        }
        let value = if on_game_thread { Self::GAME_THREAD } else { Self::OTHER_THREAD };
        if self.thread.compare_exchange(Self::UNDECIDED, value, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return;
        }
        if on_game_thread {
            curia::info!("key feed thread is the game thread");
        } else {
            curia::warn!("key injection not available: the key feed is not the game thread");
            self.state.areas().withdraw(Self::KEY_INJECT);
        }
    }
}
