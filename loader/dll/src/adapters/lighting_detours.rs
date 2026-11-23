use std::cell::Cell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_runtime::VvPlan;

use crate::{Boundary, LightingAdapter, SessionHook};

/// `BrightnessPair* BlockTessellatorCache::getLightColor(BrightnessPair* out, BlockPos const&, Block const*)`:
/// rcx the cache, rdx the 2-byte result (sky, block), r8 the position, r9 the block; the result in rax.
type TerrainLightFn = unsafe extern "C" fn(usize, *mut [u8; 2], *const [i32; 3], usize) -> usize;
/// `BrightnessPair* BlockSource::getLightColor(BrightnessPair* out, BlockPos const&, Brightness min)`: the
/// minimum is passed on as the whole register.
type EntityLightFn = unsafe extern "C" fn(usize, *mut [u8; 2], *const [i32; 3], u64) -> usize;
/// `bool LightPropagationCoordinator::propagateForMesh(BlockSource&, SubChunkPos const&, u16 variant, u16
/// flags, provider, onResult, onEmpty)`: the two words are passed on as whole registers and stack slots.
type PropagateFn = unsafe extern "C" fn(usize, usize, *const [i32; 3], u64, u64, usize, usize, usize) -> u64;
/// The light grid fill of the static light spread. 26.63 (`fillSubChunkGrid`): `(BlockSource*, SubChunkPos
/// const*, u16 mode, grid*, u32* mask)`; 70.x (`fillSubChunkGridNoMode`): `(BlockSource*, SubChunkPos const*,
/// grid*, u32*, u32* mask)`. The detour passes the five arguments on unchanged and reads only rdx, so one body
/// serves both.
type GridFillFn = unsafe extern "C" fn(usize, *const [i32; 3], u64, u64, usize) -> u64;

static TERRAIN_LIGHT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ENTITY_LIGHT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static PROPAGATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static GRID_FILL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

thread_local! {
    // Set on the game thread by the first `renderLevel` there; the entity light is raised on that thread only.
    static GAME_THREAD: Cell<bool> = const { Cell::new(false) };
    // The emitter of the `propagateForMesh` call on this thread's stack, for the grid fill below it.
    static ARMED: Cell<Option<(usize, [i32; 3], u8)>> = const { Cell::new(None) };
}

/// The loader's lighting detours. None of them logs, locks or allocates on the light getter path. A panic
/// inside the adapter gives the game its own result.
pub struct LightingDetours;

impl LightingDetours {
    pub const TERRAIN_LIGHT: &'static str = "BlockTessellatorCache.getLightColor";
    pub const ENTITY_LIGHT: &'static str = "BlockSource.getLightColor";
    pub const PROPAGATE: &'static str = "LightPropagationCoordinator.propagateForMesh";
    pub const GRID_FILL: &'static str = "LightPropagation.fillSubChunkGrid";
    pub const GRID_FILL_NO_MODE: &'static str = "LightPropagation.fillSubChunkGridNoMode";

    /// The terrain light getter hook.
    pub fn terrain_light_hook() -> SessionHook {
        Self::hook(Self::TERRAIN_LIGHT, Self::terrain_light as TerrainLightFn as *const c_void, &TERRAIN_LIGHT)
    }

    /// The `propagateForMesh` hook of the static light spread.
    pub fn propagate_hook() -> SessionHook {
        Self::hook(Self::PROPAGATE, Self::propagate as PropagateFn as *const c_void, &PROPAGATE)
    }

    pub fn entity_hook() -> SessionHook {
        Self::hook(Self::ENTITY_LIGHT, Self::entity_light as EntityLightFn as *const c_void, &ENTITY_LIGHT)
    }

    /// The grid fill hook on `entry`, one of `GRID_FILL` and `GRID_FILL_NO_MODE`.
    pub fn grid_fill_hook(entry: &'static str) -> SessionHook {
        Self::hook(entry, Self::grid_fill as GridFillFn as *const c_void, &GRID_FILL)
    }

    fn hook(entry: &'static str, detour: *const c_void, original: &'static AtomicPtr<c_void>) -> SessionHook {
        SessionHook { entry, vtable: None, detour, original }
    }

    /// Marks the calling thread as the game thread for the entity light.
    pub(crate) fn mark_game_thread() {
        GAME_THREAD.with(|flag| flag.set(true));
    }

    pub(crate) fn is_game_thread_marked() -> bool {
        GAME_THREAD.with(Cell::get)
    }

    // The original behind `slot`, as a function of type `F`.
    //
    // Safety: the hook of `slot` is installed and its target has the signature `F`.
    unsafe fn original<F: Copy>(slot: &AtomicPtr<c_void>) -> F {
        const { assert!(std::mem::size_of::<F>() == std::mem::size_of::<*mut c_void>()) };
        let address = slot.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to the detour that calls this; the caller
        // guarantees the signature, which is pointer sized.
        unsafe { std::mem::transmute_copy::<*mut c_void, F>(&address) }
    }

    unsafe extern "C" fn terrain_light(cache: usize, out: *mut [u8; 2], pos: *const [i32; 3], block: usize) -> usize {
        // SAFETY: the hook on the terrain light getter is installed, so its slot holds a `TerrainLightFn`.
        let original = unsafe { Self::original::<TerrainLightFn>(&TERRAIN_LIGHT) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(cache, out, pos, block) };
        if out.is_null() || pos.is_null() {
            return result;
        }
        LightingAdapter::with((), |adapter| {
            if !adapter.cell().is_lit() {
                return;
            }
            // SAFETY: the game passes a live position and a 2-byte result buffer, both valid for the call.
            unsafe {
                let current = (*out)[1];
                let raised = adapter.raise(*pos, current);
                if raised != current {
                    (*out)[1] = raised;
                }
            }
        });
        result
    }

    unsafe extern "C" fn entity_light(source: usize, out: *mut [u8; 2], pos: *const [i32; 3], min: u64) -> usize {
        // SAFETY: the hook on the entity light getter is installed, so its slot holds an `EntityLightFn`.
        let original = unsafe { Self::original::<EntityLightFn>(&ENTITY_LIGHT) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(source, out, pos, min) };
        if out.is_null() || pos.is_null() {
            return result;
        }
        LightingAdapter::with((), |adapter| {
            if !adapter.cell().is_lit() || !adapter.entity_on() || !Self::is_game_thread_marked() {
                return;
            }
            // SAFETY: the game passes a live position and a 2-byte result buffer, both valid for the call.
            unsafe {
                let current = (*out)[1];
                let raised = adapter.raise(*pos, current);
                if raised != current {
                    (*out)[1] = raised;
                }
            }
        });
        result
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn propagate(
        this: usize,
        source: usize,
        pos: *const [i32; 3],
        variant: u64,
        flags: u64,
        provider: usize,
        on_result: usize,
        on_empty: usize,
    ) -> u64 {
        // SAFETY: the hook on `propagateForMesh` is installed, so its slot holds a `PropagateFn`.
        let original = unsafe { Self::original::<PropagateFn>(&PROPAGATE) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let call = || unsafe { original(this, source, pos, variant, flags, provider, on_result, on_empty) };
        if pos.is_null() {
            return call();
        }
        // SAFETY: the game passes a live SubChunkPos (three i32) for the call.
        let subchunk = unsafe { *pos };
        #[allow(clippy::cast_possible_truncation)]
        let word = variant as u16;
        let plan =
            Boundary::run(VvPlan::Normal, || LightingAdapter::with(VvPlan::Normal, |a| a.vv_plan(subchunk, word)));
        match plan {
            VvPlan::Normal => call(),
            VvPlan::Skip => 0,
            VvPlan::Emit { block, position, emission } => {
                ARMED.with(|slot| slot.set(Some((block, position, emission))));
                let result = call();
                ARMED.with(|slot| slot.set(None));
                result
            }
        }
    }

    unsafe extern "C" fn grid_fill(source: usize, pos: *const [i32; 3], third: u64, fourth: u64, mask: usize) -> u64 {
        // SAFETY: the hook on the grid fill is installed, so its slot holds a `GridFillFn`.
        let original = unsafe { Self::original::<GridFillFn>(&GRID_FILL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(source, pos, third, fourth, mask) };
        // The second caller of the grid fill runs without an armed emitter, so it is passed by here.
        let Some((block, position, emission)) = ARMED.with(Cell::get) else { return result };
        if pos.is_null() {
            return result;
        }
        // SAFETY: the core passes a live SubChunkPos (three i32) for the call.
        let subchunk = unsafe { *pos };
        Boundary::run((), || {
            LightingAdapter::with((), |adapter| adapter.push_emitter(block, position, emission, subchunk));
        });
        result
    }
}
