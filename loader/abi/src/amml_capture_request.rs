use crate::AmmlShotLayers;

/// What a mod asks for in `AmmlOverlayTable::open_capture`: which image, how many frames the loader may hold
/// (`ring`), one frame every `every_nth_present` presents, and the largest frame size. A 0 limit means no
/// limit.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlCaptureRequest {
    pub size: u32,
    pub layers: AmmlShotLayers,
    pub ring: u8,
    pub _reserved: u16,
    pub every_nth_present: u32,
    pub max_width: u32,
    pub max_height: u32,
}

impl AmmlCaptureRequest {
    /// `size` of a v1 capture request.
    pub const SIZE_V1: u32 = 20;
}

const _: () = assert!(core::mem::size_of::<AmmlCaptureRequest>() == AmmlCaptureRequest::SIZE_V1 as usize);
