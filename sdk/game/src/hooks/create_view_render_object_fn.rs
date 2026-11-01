/// The shape of the `LevelRendererPlayer::createViewRenderObject` detour:
/// `(this, ret, screen_ctx, sub_client_id) -> usize`.
///
/// The game declares `ViewRenderObject createViewRenderObject(ScreenContext&, SubClientId)`, which returns the
/// object by value through a hidden return pointer. That pointer is the second argument (RDX). The sub client id is
/// a byte in R9; pass the whole register through. The function returns the hidden pointer in RAX, and the detour
/// must return the original's RAX.
pub type CreateViewRenderObjectFn = unsafe extern "C" fn(usize, usize, usize, usize) -> usize;
