use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use crate::{BlockEntityAdapter, Boundary, LightingAdapter, LoaderState, RenderCall, SessionHook, WorldRenderAdapter};

/// `void LevelRenderer::renderLevel(ScreenContext&, FrameRenderObject const&)`.
type RenderLevelFn = unsafe extern "C" fn(*mut c_void, usize, usize);

static RENDER_LEVEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// What the detour has work for besides the game's own call. Zero: the detour calls the original and nothing else.
static WANTED: AtomicU32 = AtomicU32::new(0);
// The result of the one install, once it ran.
static INSTALLED: OnceLock<bool> = OnceLock::new();

/// The loader's one detour on `LevelRenderer::renderLevel`, shared by the held light (`lighting.terrain`), the world
/// render (`render.world_rendered`, `render.world_canvas`) and the block entity ghosts (`render.block_entities`).
///
/// The detour reads one flags word. With no flag set it calls the original and does nothing else: no lock, no
/// allocation, no log. With a flag set, the work runs in this order: the lighting rebuild (before the game's frame
/// update reads the marks), the destruction of the meshes and the release of the ghost actors that wait for the render thread, the restore of the world
/// material while it is patched, the game's `renderLevel` with its own arguments, then the `WorldRendered` event. A panic inside any step is caught at the boundary and the
/// game's own call still runs.
pub struct RenderLevelDetours;

impl RenderLevelDetours {
    pub const RENDER_LEVEL: &'static str = "LevelRenderer.renderLevel";
    /// The held light runs its rebuild before the game's call.
    pub const LIGHTING: u32 = 1;
    /// A mod listens to `WorldRendered`.
    pub const LISTENER: u32 = 2;
    /// A game mesh waits for the render thread to destroy it.
    pub const MESH_DUE: u32 = 4;
    /// The world material is patched, or waits for its restore.
    pub const MATERIAL: u32 = 8;
    /// A block entity ghost's game actor waits for the render thread to release it.
    pub const GHOST_DUE: u32 = 16;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::RENDER_LEVEL,
            vtable: None,
            detour: Self::render_level as RenderLevelFn as *const c_void,
            original: &RENDER_LEVEL,
        }
    }

    /// Installs the detour once. Every later call gives the result of the first.
    pub(crate) fn install(state: &'static LoaderState) -> bool {
        *INSTALLED.get_or_init(|| Self::hook().install(state, Self::PRIORITY))
    }

    /// Sets or clears `flag`.
    pub fn set(flag: u32, on: bool) {
        if on {
            WANTED.fetch_or(flag, Ordering::AcqRel);
        } else {
            WANTED.fetch_and(!flag, Ordering::AcqRel);
        }
    }

    /// Whether `flag` is set.
    pub fn is_set(flag: u32) -> bool {
        WANTED.load(Ordering::Relaxed) & flag != 0
    }

    // The original behind the slot, as a function of type `RenderLevelFn`.
    //
    // Safety: the hook is installed and its target has the signature `RenderLevelFn`.
    unsafe fn original() -> RenderLevelFn {
        let address = RENDER_LEVEL.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to the detour that calls this; the caller
        // guarantees the signature, which is pointer sized.
        unsafe { std::mem::transmute::<*mut c_void, RenderLevelFn>(address) }
    }

    unsafe extern "C" fn render_level(this: *mut c_void, screen: usize, frame: usize) {
        // SAFETY: the hook on `LevelRenderer.renderLevel` is installed, so its slot holds a `RenderLevelFn`.
        let original = unsafe { Self::original() };
        let wanted = WANTED.load(Ordering::Relaxed);
        if wanted == 0 {
            // SAFETY: the game's own arguments, passed on unchanged.
            unsafe { original(this, screen, frame) };
            return;
        }
        if wanted & Self::LIGHTING != 0 {
            Boundary::run((), || LightingAdapter::with((), |adapter| adapter.render_level(this)));
        }
        if wanted & Self::MESH_DUE != 0 {
            Self::set(Self::MESH_DUE, false);
            Boundary::run((), || WorldRenderAdapter::with((), WorldRenderAdapter::destroy_due));
        }
        if wanted & Self::GHOST_DUE != 0 {
            Self::set(Self::GHOST_DUE, false);
            Boundary::run((), || BlockEntityAdapter::with((), BlockEntityAdapter::destroy_due));
        }
        if wanted & Self::MATERIAL != 0 {
            Boundary::run((), || {
                WorldRenderAdapter::with((), |adapter| adapter.before_level(this.expose_provenance()))
            });
        }
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, screen, frame) };
        if wanted & Self::LISTENER != 0 {
            let call = RenderCall { renderer: this.expose_provenance(), screen, frame_object: frame };
            Boundary::run((), || WorldRenderAdapter::with((), |adapter| adapter.rendered(call)));
        }
    }
}
