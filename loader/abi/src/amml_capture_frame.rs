/// One captured frame, from `AmmlOverlayTable::capture_take`: `len` bytes of BGRA8 pixels, `stride` bytes per
/// row. The pixels stay valid until the mod gives the frame back by its `present_index` or closes the stream.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlCaptureFrame {
    pub pixels: *const u8,
    pub len: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub _reserved: u32,
    pub present_index: u64,
}

const _: () = assert!(core::mem::size_of::<AmmlCaptureFrame>() == 40);
