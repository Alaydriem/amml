use std::mem::offset_of;

use crate::TexturePtrLayout;

/// Mirror of the texture list `Mesh.renderMeshTextureList` takes (148h bytes, entry notes): eight
/// `std::variant<monostate, TexturePtr, ClientTexture, ServerTexture>` of 28h bytes each (the 20h storage, then the
/// index byte), then the element count. The list is a by-value argument: at its exit the game destroys every element
/// (for a `TexturePtr`, one use of each of its two control blocks) and sets the count to 0, also on its early returns.
/// So element 0 must be a real copy that holds its own uses.
#[repr(C)]
pub struct TextureListLayout {
    pub items: [[u64; 5]; 8], // 0x000 the variants
    pub count: u64,           // 0x140
}

impl TextureListLayout {
    /// The variant index of a `mce::TexturePtr`.
    const TEXTURE_PTR: u64 = 1;

    /// A list of one `TexturePtr`; the list takes over the uses `texture` holds.
    pub fn one(texture: TexturePtrLayout) -> Self {
        // SAFETY: `TexturePtrLayout` is two `shared_ptr`, 20h bytes of plain words; its uses move into the list.
        let texture = unsafe { std::mem::transmute::<TexturePtrLayout, [u64; 4]>(texture) };
        let mut items = [[0u64; 5]; 8];
        items[0] = [texture[0], texture[1], texture[2], texture[3], Self::TEXTURE_PTR];
        Self { items, count: 1 }
    }
}

const _: () = assert!(offset_of!(TextureListLayout, count) == 0x140);
const _: () = assert!(size_of::<TextureListLayout>() == 0x148);
