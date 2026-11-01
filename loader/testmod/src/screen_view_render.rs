//! The detour on `ScreenView::render`.

use amml_sdk::DetourGuard;

use crate::PEEK;

/// The original slot of the hook.
pub static GUARD: DetourGuard = DetourGuard::new("screen_view_render");

/// The signature of the hooked function: the view and the UI render context.
pub type ScreenViewRenderFn = unsafe extern "C" fn(usize, usize);

// void ScreenView::render(UIRenderContext&). The original draws the screen; the probe then draws on top of it.
pub extern "C" fn detour(this: usize, ctx: usize) {
    let original: ScreenViewRenderFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; the arguments are the game's.
    #[allow(unsafe_code)]
    unsafe {
        original(this, ctx)
    };
    PEEK.on_render(this, ctx);
}
