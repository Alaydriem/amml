/// The shape of the `LevelRendererPlayer::setupCamera` detour: `(this, camera, float)`.
///
/// The game declares `void setupCamera(Camera&, float)`. The float arrives in xmm2 on Win64 and must reach the
/// original bit for bit.
pub type SetupCameraFn = unsafe extern "C" fn(usize, usize, f32);
