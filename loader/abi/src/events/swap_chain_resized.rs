use crate::{AmmlEvent, AmmlEventHeader};

/// The bound swap chain resized its buffers: `ok` is 1 when the resize succeeded, and `width` and `height`
/// are the size it has now.
///
/// Raised on thread `RENDER`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SwapChainResized {
    pub header: AmmlEventHeader,
    pub ok: u8,
    pub _reserved: [u8; 3],
    pub width: u32,
    pub height: u32,
}

impl AmmlEvent for SwapChainResized {
    const ID: &'static str = "overlay.swap_chain_resized";
}

const _: () = assert!(core::mem::size_of::<SwapChainResized>() == 28);
