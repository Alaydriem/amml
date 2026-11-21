use std::sync::{Arc, OnceLock};

use amml_abi::{AmmlInputBlocks, AmmlPhase, LookDelta};

use crate::{ClientAdapter, EventHub, LoaderState, LookMoveDetours, TurnFeed};

static ADAPTER: OnceLock<Arc<LookAdapter>> = OnceLock::new();

/// The look adapter over `LocalPlayer::_applyTurnDelta`: raises `LookDelta` for the player's own turns
/// and turns the view for the input service.
///
/// The game's `Vec2` is `[pitch, yaw]` in degrees, factor 1. Its y grows turning right seen from above, as
/// the API yaw does; its x grows looking up, the opposite of the API pitch and of `mRot.x`. So `LookDelta`
/// takes `yaw = y` and `pitch = -x`, and an injected turn gives the game `x = -pitch`.
pub struct LookAdapter {
    state: &'static LoaderState,
}

impl LookAdapter {
    /// Creates the adapter once per process and gives it; `None` when it already exists.
    pub fn create(state: &'static LoaderState) -> Option<Arc<LookAdapter>> {
        let adapter = Arc::new(Self { state });
        ADAPTER.set(adapter.clone()).ok().map(|()| adapter)
    }

    /// Runs `body` on the adapter; `fallback` before `create`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&LookAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// One turn of the game, `game` the `Vec2` it passed. Gives the `Vec2` the original gets, or `None`
    /// when the turn is dropped: a listener cancelled it, or an active `LOOK` lease blocks it (the event
    /// is then raised already cancelled).
    pub(crate) fn real(&self, game: [f32; 2]) -> Option<[f32; 2]> {
        let hub = self.state.events();
        let blocked = self.state.input().book().blocks_now().0 & AmmlInputBlocks::LOOK.0 != 0;
        let mut event =
            LookDelta { header: EventHub::header::<LookDelta>(AmmlPhase::BEFORE), yaw: game[1], pitch: 0.0 - game[0] };
        event.header.cancelled = u8::from(blocked);
        hub.raise(&mut event, &hub.loader_context());
        if event.header.cancelled != 0 || blocked {
            return None;
        }
        if !event.yaw.is_finite() || !event.pitch.is_finite() {
            curia::debug!("a look delta with a value that is not finite was dropped");
            return None;
        }
        Some([0.0 - event.pitch, event.yaw])
    }
}

impl TurnFeed for LookAdapter {
    fn send_turn(&self, yaw: f32, pitch: f32) -> bool {
        let Some(player) = ClientAdapter::get().and_then(ClientAdapter::primary_local_player) else {
            return false;
        };
        // SAFETY: `player` is the live local player the primary client gave on the game thread, where the
        // input service applies the frame's turn.
        unsafe { LookMoveDetours::send_turn(std::ptr::with_exposed_provenance_mut(player), 0.0 - pitch, yaw) }
    }
}
