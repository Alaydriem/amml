use crate::AmmlUseRow;

/// What a mod says it uses, reached through `AmmlModInfo::uses`.
///
/// `rows` points to `count` rows and stays valid until `amml_mod_unload` returns.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlUses {
    pub size: u32,
    pub version: u32,
    pub rows: *const AmmlUseRow,
    pub count: usize,
}

impl AmmlUses {
    /// `size` of a v1 uses block. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 24;
}

const _: () = assert!(core::mem::size_of::<AmmlUses>() == AmmlUses::SIZE_V1 as usize);
