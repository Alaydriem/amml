use crate::{AmmlEvent, AmmlEventHeader};

/// The bound swap chain is about to resize its buffers. A listener may write `width` and `height`; 0 keeps
/// what the game asked for.
///
/// Raised on thread `RENDER`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SwapChainResize {
    pub header: AmmlEventHeader,
    pub width: u32,
    pub height: u32,
    pub buffer_count: u32,
}

impl AmmlEvent for SwapChainResize {
    const ID: &'static str = "overlay.swap_chain_resize";
}

const _: () = assert!(core::mem::size_of::<SwapChainResize>() == 28);
