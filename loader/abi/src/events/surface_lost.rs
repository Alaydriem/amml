use crate::{AmmlEvent, AmmlEventHeader, AmmlSurfaceLoss};

/// The draw surface of `device_epoch` is lost: every object a mod made on its device is to be released now.
///
/// Raised on thread `RENDER`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SurfaceLost {
    pub header: AmmlEventHeader,
    pub reason: AmmlSurfaceLoss,
    pub _reserved: [u8; 7],
    pub device_epoch: u64,
}

impl AmmlEvent for SurfaceLost {
    const ID: &'static str = "overlay.surface_lost";
}

const _: () = assert!(core::mem::size_of::<SurfaceLost>() == 32);
