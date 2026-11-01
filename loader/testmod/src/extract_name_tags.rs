//! The detour on `LevelRendererPlayer::extractNameTags`.

use amml_sdk::DetourGuard;

use crate::PROBE;

/// The original slot of the hook.
pub static GUARD: DetourGuard = DetourGuard::new("extract_name_tags");

/// The signature of the hooked function: `this`, the hidden return buffer and the ScreenContext, and the buffer
/// back in RAX.
pub type ExtractNameTagsFn = unsafe extern "C" fn(usize, usize, usize) -> usize;

// NameTagRenderObjectCollection LevelRendererPlayer::extractNameTags(ScreenContext&) const, returned by value
// through `ret`. The original's RAX is returned unchanged; the probe only reads the collection.
pub extern "C" fn detour(this: usize, ret: usize, screen_ctx: usize) -> usize {
    let original: ExtractNameTagsFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; the arguments are the
    // game's own.
    #[allow(unsafe_code)]
    let result = unsafe { original(this, ret, screen_ctx) };
    PROBE.on_name_tags(ret);
    result
}
