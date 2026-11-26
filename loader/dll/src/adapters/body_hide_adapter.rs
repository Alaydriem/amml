use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlActorTarget, AmmlStatus};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::HideBook;

use crate::{ActorRenderQueueLayout, BodyHideDetours, ClientAdapter, LoaderState, SessionHook};

static ADAPTER: OnceLock<BodyHideAdapter> = OnceLock::new();

/// The `camera.hide_actor` area: while any mod wishes the local player's body hidden, the loader erases the local
/// player from the actor render queue after `LevelRendererPlayer::queueRenderEntities` and skips the draw of that
/// actor in `ActorRenderDispatcher::render`, so no body is drawn by either path.
///
/// `hide_actor` only sets a wish (`HideBook`), so it runs on any thread. The render detours may run off the game
/// thread, where the game call for the local player is not allowed, so the local player is read on the game thread:
/// at the call, and at each `FrameUpdate` Before while a wish exists. The wish ends by the mod's call, its disable and
/// the world leave (the cached player ends with it).
pub struct BodyHideAdapter {
    state: &'static LoaderState,
    book: HideBook,
    // Serializes a change of the book with the store of the detour flag, so the flag follows the last change.
    change: Mutex<()>,
    // The `LevelRendererCamera.mActorRenderQueue` offset in the renderer.
    queue_at: Option<usize>,
    // The local player of the primary client as the game thread last read it; 0 for none.
    local: AtomicUsize,
}

impl BodyHideAdapter {
    pub const AREA: &'static str = "camera.hide_actor";
    const VERSION: u32 = 1;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the queue and the dispatcher when the row resolved, offers the area when both hooks stand and logs one line.
    /// Works once per process: a second call hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id == Self::AREA)
            .collect();
        let adapter = Self {
            state,
            book: HideBook::new(),
            change: Mutex::new(()),
            queue_at: state
                .resolve("LevelRendererCamera.mActorRenderQueue")
                .ok()
                .and_then(|value| usize::try_from(value).ok()),
            local: AtomicUsize::new(0),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the body hide adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            if row.shape.is_some() {
                let refused: Vec<&'static str> = BodyHideDetours::hooks()
                    .iter()
                    .filter(|hook: &&SessionHook| !hook.install(state, Self::PRIORITY))
                    .map(|hook| hook.entry)
                    .collect();
                if !refused.is_empty() {
                    row.shape = None;
                    row.missing = refused;
                }
            }
            match row.shape {
                Some(shape) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                None => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static BodyHideAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&BodyHideAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// Whether the area is offered.
    pub fn is_offered(&self) -> bool {
        self.state.areas().is_available(Self::AREA, Self::VERSION)
    }

    // The wishes hold plain data, so a lock poisoned by a panic is recovered.
    fn change(&self) -> MutexGuard<'_, ()> {
        self.change.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The mod at `owner` wishes the body of `target` hidden (`on`) or shown. The errors are those of
    /// `AmmlCameraTable::hide_actor`.
    pub fn hide_actor(&self, owner: usize, target: AmmlActorTarget, on: bool) -> Result<(), AmmlStatus> {
        if !self.is_offered() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if target != AmmlActorTarget::LOCAL_PLAYER {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let hidden = {
            let _change = self.change();
            let hidden = self.book.set(owner, on);
            BodyHideDetours::set_hidden(hidden);
            hidden
        };
        if on {
            self.refresh_local();
        }
        curia::info!("body hide", { owner: owner, on: on, hidden: hidden });
        Ok(())
    }

    // Reads the local player on the game thread; elsewhere the last read stands.
    fn refresh_local(&self) {
        if !self.state.events().gate().on_game_thread() {
            return;
        }
        let local = ClientAdapter::get().and_then(ClientAdapter::primary_local_player).unwrap_or(0);
        self.local.store(local, Ordering::Release);
    }

    /// Whether `actor` is the local player the game thread last read.
    pub(crate) fn is_local_player(&self, actor: usize) -> bool {
        actor != 0 && actor == self.local.load(Ordering::Acquire)
    }

    /// After `queueRenderEntities` of the renderer at `renderer` while a body is hidden: the local player leaves the
    /// actor render queue. Nothing changes when it is absent, or when no local player or queue offset is known.
    pub(crate) fn erase_local_player(&self, renderer: usize) {
        let local = self.local.load(Ordering::Acquire);
        let Some(queue) = self.queue_at.and_then(|at| renderer.checked_add(at)).filter(|_| local != 0) else {
            return;
        };
        // SAFETY: `renderer` is the `this` of the `queueRenderEntities` call that just returned on this thread, so the
        // queue at the derived offset is live and nothing else uses it.
        unsafe { ActorRenderQueueLayout::erase_actor(queue, local) };
    }

    /// A `FrameUpdate` Before, on the game thread: while a body is hidden the local player is read again.
    pub(crate) fn frame_began(&self) {
        if BodyHideDetours::hidden() {
            self.refresh_local();
        }
    }

    /// Before `WorldLeft`: every wish ends and the cached player goes.
    pub(crate) fn world_left(&self) {
        self.local.store(0, Ordering::Release);
        let _change = self.change();
        if self.book.world_left() {
            BodyHideDetours::set_hidden(false);
            curia::info!("body hide cleared", { reason: "world_left" });
        }
    }

    /// The mod at `owner` is disabled: its wish ends. No game call is made, so this runs on any thread.
    pub fn disable_owner(&self, owner: usize) {
        let _change = self.change();
        if self.book.disable(owner) {
            BodyHideDetours::set_hidden(self.book.hidden());
            curia::info!("body hide cleared", { reason: "disabled", owner: owner });
        }
    }
}
