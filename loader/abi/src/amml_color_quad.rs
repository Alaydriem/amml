use crate::{AmmlRgba, AmmlVec3};

/// One quad of a world mesh: four corners in mesh space, in blocks, in drawing order, and one colour for the whole
/// quad. The loader draws one side (the corners as given) or, with `AmmlMeshStyle::two_sided`, both.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmlColorQuad {
    pub corners: [AmmlVec3; 4],
    pub rgba: AmmlRgba,
}

const _: () = assert!(core::mem::size_of::<AmmlColorQuad>() == 52);
