use crate::{AmmlEvent, AmmlEventHeader, AmmlReleaseReason};

/// The loader released an injected input of a lease.
///
/// From version 2, `input` and `code` name the one key or mouse button released, when only one was. A reader
/// checks `header.version` before it reads them: in a version 1 struct those bytes are padding.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InjectedInputReleased {
    pub header: AmmlEventHeader,
    pub lease: u64,
    pub reason: AmmlReleaseReason,
    /// From version 2: `ALL`, `KEY` or `BUTTON`.
    pub input: u8,
    /// From version 2: the key code (`KEY`) or the `AmmlMouseButton` value (`BUTTON`); 0 for `ALL`.
    pub code: u16,
    pub _reserved: u32,
}

impl InjectedInputReleased {
    /// Every injected input of the lease was released.
    pub const ALL: u8 = 0;
    /// Only the key in `code` was released.
    pub const KEY: u8 = 1;
    /// Only the mouse button in `code` was released.
    pub const BUTTON: u8 = 2;
}

impl AmmlEvent for InjectedInputReleased {
    const ID: &'static str = "input.injected_released";
    const VERSION: u32 = 2;
}

const _: () = assert!(core::mem::size_of::<InjectedInputReleased>() == 32);
