//! The detour on `LevelRendererPlayer::setupCamera`.

use amml_game::hooks::SetupCameraFn;
use amml_sdk::DetourGuard;

use crate::PROBE;

/// The original slot of the hook.
pub static GUARD: DetourGuard = DetourGuard::new("setup_camera");

// void LevelRendererPlayer::setupCamera(Camera&, float). The float is passed to the original unchanged.
pub extern "C" fn detour(this: usize, camera: usize, a: f32) {
    let original: SetupCameraFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; the arguments are the
    // game's own.
    #[allow(unsafe_code)]
    unsafe {
        original(this, camera, a)
    };
    PROBE.on_setup_camera(camera);
}
