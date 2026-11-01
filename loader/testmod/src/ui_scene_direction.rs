//! The detour on `UIScene::handleDirection`.

use amml_sdk::DetourGuard;

use crate::PEEK;

/// The original slot of the hook.
pub static GUARD: DetourGuard = DetourGuard::new("ui_scene_direction");

/// The signature of the hooked function: the scene, the `DirectionId`, the step `x` and `y`, and the `FocusImpact`.
pub type UiSceneDirectionFn = unsafe extern "C" fn(usize, u8, f32, f32, u8);

// void UIScene::handleDirection(DirectionId, float, float, FocusImpact). The original runs first and always; the probe
// then logs a wheel step.
pub extern "C" fn detour(scene: usize, direction: u8, x: f32, y: f32, focus_impact: u8) {
    let original: UiSceneDirectionFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; the arguments are the game's.
    #[allow(unsafe_code)]
    unsafe {
        original(scene, direction, x, y, focus_impact)
    };
    PEEK.on_direction(scene, direction, [x, y]);
}
