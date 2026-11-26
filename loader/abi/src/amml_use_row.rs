use crate::AmmlStr;

/// One thing a mod feature uses: an event (`kind` 0) or an area (`kind` 1) called `name`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlUseRow {
    pub feature: AmmlStr,
    pub kind: u8,
    pub name: AmmlStr,
    pub version: u32,
    pub required: u8,
}

impl AmmlUseRow {
    /// `kind` of a row that names an event.
    pub const KIND_EVENT: u8 = 0;
    /// `kind` of a row that names an area.
    pub const KIND_AREA: u8 = 1;
}

const _: () = assert!(core::mem::size_of::<AmmlUseRow>() == 48);
