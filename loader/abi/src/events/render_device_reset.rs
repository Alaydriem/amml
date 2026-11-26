use crate::{AmmlEvent, AmmlEventHeader};

/// The loader rebuilt the draw surface on a new device, `device_epoch`.
///
/// Raised on thread `RENDER`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RenderDeviceReset {
    pub header: AmmlEventHeader,
    pub device_epoch: u64,
}

impl AmmlEvent for RenderDeviceReset {
    const ID: &'static str = "overlay.render_device_reset";
}

const _: () = assert!(core::mem::size_of::<RenderDeviceReset>() == 24);
