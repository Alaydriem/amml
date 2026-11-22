use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use crate::{BodyHideAdapter, Boundary, ScreenDetours, SessionHook};

/// `void LevelRendererPlayer::queueRenderEntities(LevelRenderPreRenderUpdateParameters const&)`: rcx the renderer, rdx
/// the parameters; no result.
type QueueFn = unsafe extern "C" fn(*mut c_void, *mut c_void);
/// `void ActorRenderDispatcher::render(BaseActorRenderContext&, Actor&, Vec3 const&, Vec3 const&, Vec2 const&, bool)`:
/// rcx the dispatcher, rdx the context, r8 the actor, r9 the camera target, then three stack words (the position, the
/// rotation and the bool), all passed on as whole words; no result.
type RenderFn = unsafe extern "C" fn(usize, usize, usize, usize, usize, usize, usize);

static QUEUE_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static RENDER_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// Whether any mod wishes the local player's body hidden. False: each detour calls the original and nothing else.
static HIDDEN: AtomicBool = AtomicBool::new(false);

/// The loader's detours on `LevelRendererPlayer::queueRenderEntities` and `ActorRenderDispatcher::render`, for
/// `camera.hide_actor`.
///
/// Each detour reads one relaxed flag. With no wish to hide it calls the original and does nothing else: no lock, no
/// allocation, no log. While a mod wishes the body hidden the queue detour erases the local player from the actor
/// render queue after the game filled it, and the dispatcher detour returns without the draw when the actor is the local
/// player. A panic inside the loader's work is caught at the boundary; the game's own call runs with its own arguments.
pub struct BodyHideDetours;

impl BodyHideDetours {
    pub const QUEUE: &'static str = "LevelRendererPlayer.queueRenderEntities";
    pub const RENDERER_VTABLE: &'static str = "LevelRendererPlayer.vtable";
    pub const RENDER: &'static str = "ActorRenderDispatcher.render";

    /// The two hooks, in install order.
    pub fn hooks() -> [SessionHook; 2] {
        [
            SessionHook {
                entry: Self::QUEUE,
                vtable: Some(Self::RENDERER_VTABLE),
                detour: Self::queue as QueueFn as *const c_void,
                original: &QUEUE_ORIGINAL,
            },
            SessionHook {
                entry: Self::RENDER,
                vtable: None,
                detour: Self::render as RenderFn as *const c_void,
                original: &RENDER_ORIGINAL,
            },
        ]
    }

    /// Sets whether any mod wishes the body hidden.
    pub fn set_hidden(hidden: bool) {
        HIDDEN.store(hidden, Ordering::Release);
    }

    /// Whether the detours work for a hide: one relaxed load.
    pub fn hidden() -> bool {
        HIDDEN.load(Ordering::Relaxed)
    }

    unsafe extern "C" fn queue(renderer: *mut c_void, parameters: *mut c_void) {
        // SAFETY: the hook on `queueRenderEntities` is installed, so its slot holds a `QueueFn`.
        let original = unsafe { ScreenDetours::original::<QueueFn>(&QUEUE_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(renderer, parameters) };
        if Self::hidden() {
            let renderer = renderer.expose_provenance();
            Boundary::run((), || BodyHideAdapter::with((), |adapter| adapter.erase_local_player(renderer)));
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn render(
        dispatcher: usize,
        context: usize,
        actor: usize,
        target: usize,
        position: usize,
        rotation: usize,
        flag: usize,
    ) {
        if Self::hidden()
            && Boundary::run(false, || BodyHideAdapter::with(false, |adapter| adapter.is_local_player(actor)))
        {
            return;
        }
        // SAFETY: the hook on `ActorRenderDispatcher.render` is installed, so its slot holds a `RenderFn`.
        let original = unsafe { ScreenDetours::original::<RenderFn>(&RENDER_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(dispatcher, context, actor, target, position, rotation, flag) };
    }
}
