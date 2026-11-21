use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlLightSource, AmmlStatus};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::{EmitterEntry, LightBook, LightCell, LightChange, LightPoint, LightRefusal, VvPlan};

use crate::{BlockHandles, CoordinatorWalk, EmitterPush, LightingDetours, LoaderState, SessionHook};

static ADAPTER: OnceLock<LightingAdapter> = OnceLock::new();

/// The loader-owned held light (`lighting.terrain`), with the entity light (`lighting.entity`) and the
/// Vibrant Visuals hand emitter (`lighting.vibrant`).
///
/// Each mod has one light slot (at most `LightBook::SLOTS`); a block gets the highest value of all lights.
/// A change of a mod's light marks the reach of its old and new light for a rebuild, which the
/// `renderLevel` detour does before the frame's own update reads the marks. The light getters read the
/// lights from a `LightCell` without a lock.
pub struct LightingAdapter {
    state: &'static LoaderState,
    book: LightBook,
    cell: LightCell,
    pending: Mutex<LightChange>,
    walk: Option<CoordinatorWalk>,
    emitter: Option<EmitterPush>,
    entity: AtomicBool,
    rebuild_refused: AtomicBool,
    rebuild_logged: AtomicBool,
    thread_checked: AtomicBool,
}

impl LightingAdapter {
    pub const TERRAIN: &'static str = "lighting.terrain";
    pub const ENTITY: &'static str = "lighting.entity";
    pub const VIBRANT: &'static str = "lighting.vibrant";
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the lighting functions whose catalog shape resolved and offers each area whose hooks are in.
    /// `lighting.entity` and `lighting.vibrant` are offered only with `lighting.terrain`, whose call sets the
    /// light. Logs one `lighting adapter` line. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        let shape_of = |rows: &[Availability], id: &str| rows.iter().find(|row| row.id == id).and_then(|row| row.shape);
        let install = |hook: &SessionHook| hook.install(state, Self::PRIORITY);
        let terrain = shape_of(&rows, Self::TERRAIN).is_some() && LightingDetours::terrain_hooks().iter().all(install);
        let walk = terrain.then(|| CoordinatorWalk::new(state)).flatten();
        let terrain = terrain && walk.is_some();
        let entity = terrain && shape_of(&rows, Self::ENTITY).is_some() && install(&LightingDetours::entity_hook());
        let vibrant_shape = shape_of(&rows, Self::VIBRANT).filter(|_| terrain);
        let emitter = vibrant_shape.and_then(|shape| {
            let entry = if shape == AdapterCatalog::LIGHT_GRID_NO_MODE_SHAPE {
                LightingDetours::GRID_FILL_NO_MODE
            } else {
                LightingDetours::GRID_FILL
            };
            let push = EmitterPush::new(state)?;
            install(&LightingDetours::grid_fill_hook(entry)).then_some(push)
        });
        let vibrant = emitter.is_some();
        for row in &mut rows {
            let on = match row.id {
                Self::TERRAIN => terrain,
                Self::ENTITY => entity,
                _ => vibrant,
            };
            if on {
                state.areas().offer(row.id, row.version);
            } else {
                row.shape = None;
            }
        }
        curia::info!("lighting adapter", {
            terrain: if terrain { "on" } else { "off" },
            entity: if entity { "on" } else { "off" },
            vibrant: vibrant_shape.filter(|_| vibrant).unwrap_or("off")
        });
        let adapter = Self {
            state,
            book: LightBook::new(),
            cell: LightCell::new(),
            pending: Mutex::new(LightChange::default()),
            walk,
            emitter,
            entity: AtomicBool::new(entity),
            rebuild_refused: AtomicBool::new(false),
            rebuild_logged: AtomicBool::new(false),
            thread_checked: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the lighting adapter is already installed");
        }
        rows
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&LightingAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn is_own_row(row: &Availability) -> bool {
        row.kind == NeedKind::Area && [Self::TERRAIN, Self::ENTITY, Self::VIBRANT].contains(&row.id)
    }

    pub(crate) fn cell(&self) -> &LightCell {
        &self.cell
    }

    pub(crate) fn entity_on(&self) -> bool {
        self.entity.load(Ordering::Relaxed)
    }

    // A poisoned lock is recovered: the pending boxes are plain data.
    fn pending(&self) -> MutexGuard<'_, LightChange> {
        self.pending.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Sets (or with `None` clears) the light of the mod at `owner`. The caller checked the thread, the gate
    /// and the area. A level outside 1 to 15 or a position that is not finite gives `INVALID_ARGUMENT`; a
    /// stale block handle `STALE`; a fifth mod `REFUSED`.
    pub(crate) fn set(&self, owner: usize, source: Option<&AmmlLightSource>) -> Result<(), AmmlStatus> {
        let point = match source {
            None => None,
            Some(source) => {
                let block = if source.has_block != 0 { BlockHandles::address(self.state, source.block)? } else { 0 };
                let point = LightPoint::at(
                    [source.position.x, source.position.y, source.position.z],
                    source.level,
                    source.falloff,
                    block,
                );
                Some(point.ok_or(AmmlStatus::INVALID_ARGUMENT)?)
            }
        };
        let change =
            self.book.set(owner, point, |points| self.cell.publish(points)).map_err(|refusal| match refusal {
                LightRefusal::Full => AmmlStatus::REFUSED,
            })?;
        self.pending().join(change);
        Ok(())
    }

    /// Clears the light of the disabled mod at `owner`, so its lit area goes dark at the next frame.
    pub(crate) fn clear_owner(&self, owner: usize, module: &str) {
        if !self.book.holds(owner) {
            return;
        }
        if let Ok(change) = self.book.set(owner, None, |points| self.cell.publish(points)) {
            self.pending().join(change);
        }
        curia::info!("lighting cleared", { mod_name: module });
    }

    /// Before a world is left: every light goes out and no rebuild is kept for the old world.
    pub(crate) fn world_left(&self) {
        let _ = self.book.clear_all(|points| self.cell.publish(points));
        *self.pending() = LightChange::default();
    }

    /// Before the game's `renderLevel`: checks the thread once, then rebuilds the marked reach.
    pub(crate) fn render_level(&self, level_renderer: *mut c_void) {
        self.check_thread();
        let change = std::mem::take(&mut *self.pending());
        if change.is_empty() || self.rebuild_refused.load(Ordering::Relaxed) {
            return;
        }
        let Some(walk) = &self.walk else { return };
        // SAFETY: `level_renderer` is the `this` the game passed to `renderLevel`, on the thread that runs
        // this frame's update, which reads the marks.
        match unsafe { walk.mark(level_renderer, &change.boxes) } {
            Ok(calls) => {
                if !self.rebuild_logged.swap(true, Ordering::Relaxed) {
                    curia::info!("lighting rebuild", { boxes: change.boxes.len(), calls: calls });
                }
            }
            Err(reason) => {
                self.rebuild_refused.store(true, Ordering::Relaxed);
                curia::warn!("lighting rebuild refused", { reason: reason });
            }
        }
    }

    // The entity light is raised on the game thread only. `renderLevel` on the game thread marks it; on
    // another thread (once the gate is open, so the game thread is known) the entity light is withdrawn.
    fn check_thread(&self) {
        if LightingDetours::is_game_thread_marked() || self.thread_checked.load(Ordering::Relaxed) {
            return;
        }
        let gate = self.state.events().gate();
        if !gate.is_open() {
            return;
        }
        self.thread_checked.store(true, Ordering::Relaxed);
        if gate.on_game_thread() {
            LightingDetours::mark_game_thread();
            curia::info!("lighting render thread", { game: true });
        } else {
            curia::info!("lighting render thread", { game: false });
            if self.entity.swap(false, Ordering::Relaxed) {
                self.state.areas().withdraw(Self::ENTITY);
            }
        }
    }

    /// The plan of the Vibrant Visuals static light pass for one subchunk.
    pub(crate) fn vv_plan(&self, subchunk: [i32; 3], variant: u16) -> VvPlan {
        self.cell.vv_plan(subchunk, variant, self.emitter.is_some())
    }

    /// After the grid fill under an armed `propagateForMesh`: adds the hand emitter to this thread's emitter
    /// vector when its block is inside the pass window.
    pub(crate) fn push_emitter(&self, block: usize, position: [i32; 3], emission: u8, subchunk: [i32; 3]) {
        let (Some(push), Some(local)) = (&self.emitter, EmitterEntry::local(position, subchunk)) else { return };
        if block == 0 {
            return;
        }
        // SAFETY: called from the grid fill detour inside the light spread core on this thread, which owns
        // its emitter vector in the game's thread-local block.
        unsafe { push.push(&EmitterEntry::bytes(block, local, emission)) };
    }
}
