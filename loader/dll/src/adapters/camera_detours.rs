use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use crate::{Boundary, CameraAdapter, SessionHook};

/// `void LevelRendererPlayer::setupCamera(mce::Camera&, float)`: rcx the player, rdx the camera, xmm2 the frame alpha.
type SetupCameraFn = unsafe extern "C" fn(*mut c_void, *mut c_void, f32);
/// `ViewRenderObject LevelRendererPlayer::createViewRenderObject(ScreenContext&, SubClientId)`: rcx the player, rdx the
/// hidden out pointer, r8 the screen context, r9b the sub client id (passed on as the whole register); the out pointer
/// comes back in rax.
type CreateViewFn = unsafe extern "C" fn(*mut c_void, usize, usize, usize) -> usize;
/// `void LevelRendererCamera::preRenderUpdate(ScreenContext&, LevelRenderPreRenderUpdateParameters&)`: rcx the level
/// camera, rdx the screen context, r8 the parameters.
type PreRenderFn = unsafe extern "C" fn(*mut c_void, usize, usize);

static SETUP_CAMERA: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static CREATE_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static PRE_RENDER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// What the detours have work for besides the game's own call. Zero: each detour calls the original and nothing else.
static WANTED: AtomicU32 = AtomicU32::new(0);

/// The loader's detours on `LevelRendererPlayer::setupCamera`, `createViewRenderObject` and
/// `LevelRendererCamera::preRenderUpdate`.
///
/// Each detour reads one flags word. With no flag set it calls the original and does nothing else: no lock, no
/// allocation, no log. The `setupCamera` detour runs its work while a mod listens to `CameraSetup` or a pose is held:
/// `CameraSetup` Before, the game's call, `CameraSetup` After, then the rotation write. The other two run their work
/// while a pose is held: the game's call, then the position write. A panic inside the loader's work is caught at the
/// boundary; the game's own call runs once with its own arguments.
pub struct CameraDetours;

impl CameraDetours {
    pub const SETUP_CAMERA: &'static str = "LevelRendererPlayer.setupCamera";
    pub const CREATE_VIEW: &'static str = "LevelRendererPlayer.createViewRenderObject";
    pub const PRE_RENDER: &'static str = "LevelRendererCamera.preRenderUpdate";
    /// A mod listens to `CameraSetup`.
    pub const LISTENER: u32 = 1;
    /// A pose of the current camera frame is held.
    pub const POSE: u32 = 2;

    pub fn setup_hook() -> SessionHook {
        SessionHook {
            entry: Self::SETUP_CAMERA,
            vtable: None,
            detour: Self::setup_camera as SetupCameraFn as *const c_void,
            original: &SETUP_CAMERA,
        }
    }

    /// The two position hooks, in install order.
    pub fn position_hooks() -> [SessionHook; 2] {
        [
            SessionHook {
                entry: Self::CREATE_VIEW,
                vtable: None,
                detour: Self::create_view as CreateViewFn as *const c_void,
                original: &CREATE_VIEW,
            },
            SessionHook {
                entry: Self::PRE_RENDER,
                vtable: None,
                detour: Self::pre_render as PreRenderFn as *const c_void,
                original: &PRE_RENDER,
            },
        ]
    }

    /// Sets or clears `flag`.
    pub fn set(flag: u32, on: bool) {
        if on {
            WANTED.fetch_or(flag, Ordering::AcqRel);
        } else {
            WANTED.fetch_and(!flag, Ordering::AcqRel);
        }
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

    unsafe extern "C" fn setup_camera(this: *mut c_void, camera: *mut c_void, alpha: f32) {
        // SAFETY: the hook on `LevelRendererPlayer.setupCamera` is installed, so its slot holds a `SetupCameraFn`.
        let original = unsafe { Self::original::<SetupCameraFn>(&SETUP_CAMERA) };
        if WANTED.load(Ordering::Relaxed) == 0 {
            // SAFETY: the game's own arguments, passed on unchanged.
            unsafe { original(this, camera, alpha) };
            return;
        }
        let camera_address = camera.expose_provenance();
        let frame = Boundary::run(None, || CameraAdapter::with(None, |adapter| adapter.before_setup(camera_address)));
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, camera, alpha) };
        if let Some(frame) = frame {
            Boundary::run((), || CameraAdapter::with((), |adapter| adapter.after_setup(frame, camera_address)));
        }
    }

    unsafe extern "C" fn create_view(this: *mut c_void, out: usize, screen: usize, sub_client: usize) -> usize {
        // SAFETY: the hook on `LevelRendererPlayer.createViewRenderObject` is installed, so its slot holds a
        // `CreateViewFn`.
        let original = unsafe { Self::original::<CreateViewFn>(&CREATE_VIEW) };
        // SAFETY: the game's own arguments, passed on unchanged.
        let result = unsafe { original(this, out, screen, sub_client) };
        if WANTED.load(Ordering::Relaxed) & Self::POSE != 0 {
            Boundary::run((), || CameraAdapter::with((), |adapter| adapter.after_view_object(out)));
        }
        result
    }

    unsafe extern "C" fn pre_render(this: *mut c_void, screen: usize, params: usize) {
        // SAFETY: the hook on `LevelRendererCamera.preRenderUpdate` is installed, so its slot holds a `PreRenderFn`.
        let original = unsafe { Self::original::<PreRenderFn>(&PRE_RENDER) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, screen, params) };
        if WANTED.load(Ordering::Relaxed) & Self::POSE != 0 {
            let level_camera = this.expose_provenance();
            Boundary::run((), || CameraAdapter::with((), |adapter| adapter.after_pre_render(level_camera)));
        }
    }
}
