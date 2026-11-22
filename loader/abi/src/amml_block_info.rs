/// What `WorldBlockInfoFn` reads of a block: the id of its block type (a number of `WorldBlockTypeNameFn`; 0 when the
/// game gave no readable type), whether it is air, and whether its type is interactive (`isInteractiveBlock`; 0 for
/// air).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlBlockInfo {
    pub type_id: u32,
    pub is_air: u8,
    pub is_interactive: u8,
    pub _reserved: [u8; 2],
}

const _: () = assert!(core::mem::size_of::<AmmlBlockInfo>() == 8);
