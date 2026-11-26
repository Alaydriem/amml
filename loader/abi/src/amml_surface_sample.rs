use crate::AmmlBlockPos;

/// One surface of a column that `scan_surfaces` found: the block that gives the support, and the raw light of the block
/// above it (0 to 15 each, no sky darken).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlSurfaceSample {
    pub pos: AmmlBlockPos,
    pub block_light: u8,
    pub sky_light: u8,
    pub _reserved: [u8; 2],
}

const _: () = assert!(core::mem::size_of::<AmmlSurfaceSample>() == 16);
