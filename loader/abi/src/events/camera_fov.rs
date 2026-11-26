use crate::{AmmlEvent, AmmlEventHeader};

/// The game asks for the camera field of view, in degrees. `base` is the game's value for this call;
/// `steady` is the same value without variable FOV (sprint, speed effects), equal to `base` when the
/// game asked without it. `result` starts as `base`; the game gets what the listeners leave in it when
/// it is finite and between 0 and 180 (both excluded), else `base`.
///
/// `result` is the only field a listener can change: the loader writes `base`, `steady` and `_reserved` (0)
/// back after every listener. `_reserved` stays reserved; a later version appends its fields after it, so
/// its size is 36 or more.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CameraFov {
    pub header: AmmlEventHeader,
    pub base: f32,
    pub steady: f32,
    pub result: f32,
    pub _reserved: u32,
}

impl AmmlEvent for CameraFov {
    const ID: &'static str = "camera.fov";
}

const _: () = assert!(core::mem::size_of::<CameraFov>() == 32);
