use crate::{AmmlEvent, AmmlEventHeader};

/// The bound swap chain presented frame `frame_id`: `result` is the `HRESULT` of the present and `cpu_us` the
/// time the call took, in microseconds.
///
/// Raised on thread `RENDER`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FramePresented {
    pub header: AmmlEventHeader,
    pub frame_id: u64,
    pub result: i32,
    pub cpu_us: u32,
}

impl AmmlEvent for FramePresented {
    const ID: &'static str = "overlay.frame_presented";
}

const _: () = assert!(core::mem::size_of::<FramePresented>() == 32);
