/// Mirror of the game's `mce::Mesh` on 26.63 and 70.26 (260h bytes, pointer aligned). Opaque: the loader only makes,
/// draws and destroys meshes through the game's functions and never reads a member, so it needs the size. The
/// `Mesh.size` entry must have this value.
#[repr(C)]
pub struct MeshLayout {
    _bytes: [u64; 76],
}

impl MeshLayout {
    /// The size of the mesh, the value the `Mesh.size` entry has for this mirror.
    pub const SIZE: u64 = size_of::<MeshLayout>() as u64;
}

const _: () = assert!(size_of::<MeshLayout>() == 0x260);
const _: () = assert!(align_of::<MeshLayout>() == 8);
