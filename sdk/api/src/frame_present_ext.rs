use amml_abi::FramePresent;

use crate::DrawSurface;
use crate::lent_surfaces::LentSurfaces;

/// Typed reads of a `FramePresent`.
pub trait FramePresentExt {
    /// The surface to draw on: `None` while the loader has none (no device yet, a 0 x 0 chain, the D3D12
    /// path waiting for its queue), and for any `FramePresent` that is not the event of a running
    /// listener call on this thread (a copy, or one kept past its call). It is the surface the loader
    /// wrote before the call: a listener's writes to `surface` do not change it.
    fn surface(&self) -> Option<DrawSurface<'_>>;
}

impl FramePresentExt for FramePresent {
    fn surface(&self) -> Option<DrawSurface<'_>> {
        let lent = LentSurfaces::find(std::ptr::from_ref(self))?;
        // SAFETY: the surface was lent for this event's running call on this thread, and the borrow of
        // `self` ends inside that call; the loader keeps the surface pointers live for the call.
        #[allow(unsafe_code)]
        unsafe {
            DrawSurface::new(lent.surface, lent.copy_back_buffer)
        }
    }
}
