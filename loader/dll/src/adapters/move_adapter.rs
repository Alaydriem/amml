use std::sync::OnceLock;

use amml_abi::{AmmlInputBlocks, AmmlMoveIntent, AmmlPhase, PlayerMoveInput};

use crate::{EventHub, LoaderState, MoveLayout};

static ADAPTER: OnceLock<MoveAdapter> = OnceLock::new();

/// The move adapter over `ClientInputUpdateSystem::extractRawHIDInput`, after the game filled the
/// player's `RawMoveInputComponent` from its keyboard.
///
/// In order: an active `MOVE` lease clears the move fields, the move intent of the leases is written
/// (edge bits untouched), `PlayerMoveInput` is raised with the move read from the component
/// (`injected = 1` when an intent was written), and an intent a listener changed is written back.
pub struct MoveAdapter {
    state: &'static LoaderState,
    layout: MoveLayout,
}

impl MoveAdapter {
    /// Creates the adapter once per process with `layout`; false when it already exists.
    pub fn create(state: &'static LoaderState, layout: MoveLayout) -> bool {
        ADAPTER.set(Self { state, layout }).is_ok()
    }

    /// Runs `body` on the adapter; `fallback` before `create`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&MoveAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// One fill of the component at `raw`, the non-null `RawMoveInputComponent&` the game passed.
    pub(crate) fn real(&self, raw: usize) {
        let component = std::ptr::with_exposed_provenance_mut::<[u8; MoveLayout::COMPONENT_SIZE]>(raw);
        // SAFETY: `raw` is the live component the game passed for this call, on the game thread, and it is
        // `COMPONENT_SIZE` bytes; a byte array has no alignment need. No other reference to it exists
        // while the loader holds this one.
        let component = unsafe { &mut *component };
        let input = self.state.input();
        if input.book().blocks_now().0 & AmmlInputBlocks::MOVE.0 != 0 {
            self.layout.zero(component);
        }
        let injected = input.move_intent();
        if let Some(intent) = injected {
            self.layout.write(component, intent);
        }
        let seen = self.layout.read(component);
        let mut event = PlayerMoveInput { header: EventHub::header::<PlayerMoveInput>(AmmlPhase::AFTER), intent: seen };
        event.header.injected = u8::from(injected.is_some());
        let hub = self.state.events();
        hub.raise(&mut event, &hub.loader_context());
        if event.intent != seen {
            if Self::finite(event.intent) {
                self.layout.write(component, event.intent);
            } else {
                curia::debug!("a move intent with a value that is not finite was dropped");
            }
        }
    }

    fn finite(intent: AmmlMoveIntent) -> bool {
        intent.forward.is_finite() && intent.left.is_finite()
    }
}
