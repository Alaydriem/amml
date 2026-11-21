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

impl AmmlEvent for FramePresent {
    const ID: &'static str = "overlay.frame_present";
}

const _: () = assert!(core::mem::size_of::<FramePresent>() == 96);
