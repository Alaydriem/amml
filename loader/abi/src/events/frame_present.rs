use crate::{AmmlDrawSurface, AmmlEvent, AmmlEventHeader};

/// The bound swap chain is about to present frame `frame_id`. Listeners draw on `surface`; the loader saves
/// the D3D11 pipeline state before the first listener and restores it after the last.
///
/// The restored state is: the render targets and depth view, viewports and scissor rectangles, the rasterizer,
/// blend (factor and mask) and depth-stencil (reference) states, the vertex, pixel, geometry, hull and domain
/// shaders, pixel shader resource view 0 and sampler 0, vertex and pixel constant buffer 0, and the input
/// assembler (topology, index buffer, vertex buffer 0, input layout). On a surface with `wrapped` 0 the context is
/// the game's own: a listener that changes any other state (other slots, compute, unordered access views,
/// stream-out, predication) restores it itself before it returns. Each listener sets every state it uses. The
/// surface fields are written back after each listener.
///
/// A listener at `ON_DEMAND_VERSION` gets the surface on demand: `device`, `width`, `height`, `format`, `wrapped` and
/// `device_epoch` are set, and `context`, `target` and `back_buffer` are null until it calls
/// `AmmlOverlayTable::prepare_surface`, which makes the surface (and saves the state) and gives the full surface; the
/// later listeners of the frame then get it in the event. A frame where no listener asks makes no surface. While any
/// listener of the event is older, or a capture is due, every listener gets the full surface before the first one
/// runs, and `prepare_surface` gives that surface (`OK`). On the D3D12 path, work a listener puts on `device` in a
/// frame where no surface is made is sent to the GPU at a later flush, not at the end of that frame.
///
/// Raised on thread `RENDER`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FramePresent {
    pub header: AmmlEventHeader,
    pub frame_id: u64,
    pub sync_interval: u32,
    pub flags: u32,
    pub surface: AmmlDrawSurface,
}

impl FramePresent {
    /// The listen version that takes the surface on demand (`AmmlOverlayTable::prepare_surface`).
    pub const ON_DEMAND_VERSION: u32 = 2;
}

impl AmmlEvent for FramePresent {
    const ID: &'static str = "overlay.frame_present";
}

const _: () = assert!(core::mem::size_of::<FramePresent>() == 96);
