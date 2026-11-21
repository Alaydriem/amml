use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use amml_abi::{AmmlEvent, AmmlPhase, AmmlTextureReload, AmmlThread, TexturesReloaded};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{EventHub, LoaderState, TextureAdapter, TextureReloadDetours, ThreadNote};

static ADAPTER: OnceLock<TextureReloadAdapter> = OnceLock::new();

/// The `ui.textures_reloaded` event: after the game applied the active resource pack stack the mod textures are bound
/// again (`RESOURCE_PACK`), after a render device reset they were freed (`DEVICE_RESET`); either way the mods are told.
///
/// The pack change comes from the detour on `MinecraftGame.applyActiveResourcePacks`, the device reset from
/// `TextureAdapter::device_reset`. On the game thread the event is raised at once; from another thread the reason is
/// kept and raised once, on the game thread, after the next `FrameUpdate` After listeners.
pub struct TextureReloadAdapter {
    state: &'static LoaderState,
    // The reasons seen off the game thread and not raised yet, one bit per reason value.
    pending: AtomicU8,
    pack_thread: ThreadNote,
}

impl TextureReloadAdapter {
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // The reasons in the order a flush raises them.
    const REASONS: [AmmlTextureReload; 2] = [AmmlTextureReload::DEVICE_RESET, AmmlTextureReload::RESOURCE_PACK];

    /// Hooks `MinecraftGame.applyActiveResourcePacks` when the catalog shape of `ui.textures_reloaded` resolved, and
    /// offers the event when the hook is in. Returns the event's row; a refused hook leaves the event not offered and
    /// names the entry as missing. Logs one line. Works once per process: a second call hooks nothing.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Event && row.id == TexturesReloaded::ID)
            .collect();
        let adapter = Self { state, pending: AtomicU8::new(0), pack_thread: ThreadNote::new() };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the texture reload adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            if row.shape.is_some() {
                if TextureReloadDetours::hook().install(state, Self::PRIORITY) {
                    state.events().offer(row.id, row.version);
                } else {
                    row.shape = None;
                    row.missing = vec![TextureReloadDetours::APPLY];
                }
            }
            match row.shape {
                Some(shape) => curia::info!("event offered", { event: row.id, shape: shape }),
                None => curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&TextureReloadAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    /// After the game applied the active resource pack stack: every mod texture is bound again, then
    /// `TexturesReloaded` with `RESOURCE_PACK`.
    pub(crate) fn pack_changed(&self) {
        let game = self.hub().gate().on_game_thread();
        if self.pack_thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("resource pack change thread", { game: game });
        }
        TextureAdapter::with((), |adapter| adapter.rebind("resource_pack"));
        self.note(AmmlTextureReload::RESOURCE_PACK);
    }

    /// After the texture adapter freed the textures at a render device reset: `TexturesReloaded` with
    /// `DEVICE_RESET`.
    pub(crate) fn device_reset(&self) {
        self.note(AmmlTextureReload::DEVICE_RESET);
    }

    // Raises `reason` at once on the game thread; keeps it for `flush_pending` on another thread.
    fn note(&self, reason: AmmlTextureReload) {
        if self.hub().gate().on_game_thread() {
            self.raise(reason);
        } else {
            self.pending.fetch_or(1 << reason.0, Ordering::AcqRel);
        }
    }

    /// Raises each reason kept from another thread, once. Called on the game thread after the `FrameUpdate` After
    /// listeners; with nothing kept it is one relaxed load.
    pub(crate) fn flush_pending(&self) {
        if self.pending.load(Ordering::Relaxed) == 0 {
            return;
        }
        let pending = self.pending.swap(0, Ordering::AcqRel);
        for reason in Self::REASONS {
            if pending & (1 << reason.0) != 0 {
                self.raise(reason);
            }
        }
    }

    fn raise(&self, reason: AmmlTextureReload) {
        let hub = self.hub();
        if !hub.has_listeners(TexturesReloaded::ID) {
            return;
        }
        let header = EventHub::header::<TexturesReloaded>(AmmlPhase::AFTER);
        let mut event = TexturesReloaded { header, reason, _reserved: [0; 7] };
        hub.raise(&mut event, &hub.loader_context());
    }
}
