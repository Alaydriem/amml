use std::sync::OnceLock;

use amml_abi::{AmmlInputBlocks, AmmlPhase, GamepadPoll};

use crate::{EventHub, LoaderState};

static ADAPTER: OnceLock<GamepadAdapter> = OnceLock::new();

/// The gamepad adapter over `GameControllerHandler_GameCore::refresh`, the game's only gamepad poll:
/// raises `GamepadPoll`. A skipped poll leaves the game with the state of the last poll that ran, so
/// the first poll after a block reports only the changes since then.
pub struct GamepadAdapter {
    state: &'static LoaderState,
}

impl GamepadAdapter {
    /// Creates the adapter once per process; false when it already exists.
    pub fn create(state: &'static LoaderState) -> bool {
        ADAPTER.set(Self { state }).is_ok()
    }

    /// Runs `body` on the adapter; `fallback` before `create`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&GamepadAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// One poll. True when the game must skip it: a listener cancelled it, or an active `GAMEPAD` lease
    /// blocks it (the event is then raised already cancelled).
    pub(crate) fn real(&self) -> bool {
        let hub = self.state.events();
        let blocked = self.state.input().book().blocks_now().0 & AmmlInputBlocks::GAMEPAD.0 != 0;
        let mut event = GamepadPoll { header: EventHub::header::<GamepadPoll>(AmmlPhase::BEFORE) };
        event.header.cancelled = u8::from(blocked);
        hub.raise(&mut event, &hub.loader_context());
        blocked || event.header.cancelled != 0
    }
}
