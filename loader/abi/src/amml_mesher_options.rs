/// How the block mesher builds a grid: `alpha` is written into every vertex colour; with `skip_matching_world` (0 or 1)
/// a cell whose world block is the same game block is not drawn and its index is listed; `water_tint` (0 or 1) is read
/// from area `render.water_tint` on.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlMesherOptions {
    pub alpha: u8,
    pub skip_matching_world: u8,
    pub water_tint: u8,
    pub _reserved: u8,
}

const _: () = assert!(core::mem::size_of::<AmmlMesherOptions>() == 4);
