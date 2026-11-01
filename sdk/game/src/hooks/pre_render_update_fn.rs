/// The shape of the `LevelRendererCamera::preRenderUpdate` detour: `(this, screen_ctx, params)`.
///
/// The game declares `void preRenderUpdate(ScreenContext&, LevelRenderPreRenderUpdateParameters&)`. The hook is on
/// the base function, which `LevelRendererPlayer::preRenderUpdate` calls first; a writer of the camera position
/// runs after the original returns.
pub type PreRenderUpdateFn = unsafe extern "C" fn(usize, usize, usize);
