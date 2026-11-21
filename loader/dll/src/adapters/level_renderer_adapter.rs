use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::{AmmlEvent, AmmlPhase, AmmlThread, CameraFov, DimensionChanging};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{Boundary, EventHub, LevelRendererDetours, LoaderState, SessionHook, TextureAdapter, ThreadNote};

static ADAPTER: OnceLock<LevelRendererAdapter> = OnceLock::new();

/// The level renderer adapter over `LevelRendererPlayer`: raises `CameraFov` from `getFov` and
/// `DimensionChanging` after `onWillChangeDimension`.
///
/// `CameraFov` is raised only when a mod listens to it. Its `steady` value comes from a second call of the
/// original with variable FOV off, made only when the game asked with variable FOV on. `DimensionChanging`
/// is raised at once on the game thread; from another thread it waits for the next `FrameUpdate` After.
pub struct LevelRendererAdapter {
    state: &'static LoaderState,
    // A dimension change seen off the game thread and not raised yet.
    pending: AtomicBool,
    fov_thread: ThreadNote,
    bad_fov_logged: AtomicBool,
    dimension_thread: ThreadNote,
}

impl LevelRendererAdapter {
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // The open interval a `CameraFov` result must be in, in degrees.
    const MIN_FOV: f32 = 0.0;
    const MAX_FOV: f32 = 180.0;

    /// Hooks each of the two functions whose catalog shape resolved and offers its event. Returns the
    /// `camera.fov` and `world.dimension_changing` rows; an event whose hook was refused is not offered and
    /// names its entry as missing. Logs `event offered` or `event not available` for each. Works once per
    /// process: a second call hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        let adapter = Self {
            state,
            pending: AtomicBool::new(false),
            fov_thread: ThreadNote::new(),
            bad_fov_logged: AtomicBool::new(false),
            dimension_thread: ThreadNote::new(),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the level renderer adapter is already installed");
            return rows;
        }
        let hooks = LevelRendererDetours::hooks();
        for row in &mut rows {
            if row.shape.is_some() {
                let entry = Self::hook_entry(row.id);
                let installed = entry.is_some_and(|entry| {
                    hooks.iter().find(|hook| hook.entry == entry).is_some_and(|hook| Self::hook(state, hook))
                });
                if installed {
                    state.events().offer(row.id, row.version);
                } else {
                    row.shape = None;
                    row.missing = entry.into_iter().collect();
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
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&LevelRendererAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn is_own_row(row: &Availability) -> bool {
        row.kind == NeedKind::Event && Self::hook_entry(row.id).is_some()
    }

    // The function whose detour raises `event`.
    fn hook_entry(event: &str) -> Option<&'static str> {
        match event {
            CameraFov::ID => Some(LevelRendererDetours::GET_FOV),
            DimensionChanging::ID => Some(LevelRendererDetours::WILL_CHANGE_DIMENSION),
            _ => None,
        }
    }

    fn hook(state: &'static LoaderState, hook: &SessionHook) -> bool {
        hook.install(state, Self::PRIORITY)
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    /// One `getFov` call of the game, `variable` its variable FOV bool. `original(v)` calls the game's
    /// original with variable FOV `v`. Without a `camera.fov` listener the original runs once and its value
    /// is the result. Else the original runs with variable FOV off first (`steady`), then, when `variable`
    /// is on, with the game's own arguments (`base`), and `CameraFov` is raised. The game gets `result`
    /// when it is finite and inside (0, 180), else `base`.
    pub(crate) fn fov(&self, variable: bool, mut original: impl FnMut(bool) -> f32) -> f32 {
        if !self.hub().has_listeners(CameraFov::ID) {
            return original(variable);
        }
        let steady = variable.then(|| original(false));
        let base = original(variable);
        let steady = steady.unwrap_or(base);
        let result = Boundary::run(base, || self.raise_fov(base, steady));
        if result.is_finite() && result > Self::MIN_FOV && result < Self::MAX_FOV {
            return result;
        }
        if !self.bad_fov_logged.swap(true, Ordering::AcqRel) {
            curia::debug!("a camera fov result out of range gave the game its own value", {
                result: result.to_string(),
                base: base.to_string()
            });
        }
        base
    }

    // Raises `CameraFov` with `base` and `steady`, which no listener changes, and gives the `result` the
    // listeners left.
    fn raise_fov(&self, base: f32, steady: f32) -> f32 {
        let hub = self.hub();
        let thread = if hub.gate().on_game_thread() { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.fov_thread.changed(thread) {
            let name = if thread == AmmlThread::GAME { "game" } else { "render" };
            curia::info!("camera fov thread", { thread: name });
        }
        let mut event = CameraFov {
            header: EventHub::header::<CameraFov>(AmmlPhase::AFTER),
            base,
            steady,
            result: base,
            _reserved: 0,
        };
        let keep = |event: &mut CameraFov| {
            event.base = base;
            event.steady = steady;
            event._reserved = 0;
        };
        hub.raise_keeping(&mut event, &hub.context_on(thread), &keep);
        event.result
    }

    /// After the game's `onWillChangeDimension`: `DimensionChanging` at once on the game thread; on another
    /// thread it is kept for `flush_pending`.
    pub(crate) fn will_change_dimension(&self) {
        let game = self.hub().gate().on_game_thread();
        let thread = if game { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.dimension_thread.changed(thread) {
            curia::info!("dimension change thread", { game: game });
        }
        if game {
            self.raise_dimension();
        } else {
            self.pending.store(true, Ordering::Release);
        }
    }

    /// Raises the `DimensionChanging` a dimension change off the game thread left, once. Called on the game
    /// thread after the `FrameUpdate` After listeners.
    pub(crate) fn flush_pending(&self) {
        if self.pending.swap(false, Ordering::AcqRel) {
            self.raise_dimension();
        }
    }

    /// Forgets a dimension change kept for `flush_pending`. Called before `WorldLeft`, so a change seen off
    /// the game thread in the frame the world is left does not reach the mods after the world is gone.
    pub(crate) fn drop_pending(&self) {
        self.pending.store(false, Ordering::Release);
    }

    fn raise_dimension(&self) {
        TextureAdapter::with((), |adapter| adapter.rebind("dimension_changed"));
        let hub = self.hub();
        let mut event = DimensionChanging { header: EventHub::header::<DimensionChanging>(AmmlPhase::AFTER) };
        hub.raise(&mut event, &hub.loader_context());
    }
}
