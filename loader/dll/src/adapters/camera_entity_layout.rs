use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of one element of `CameraRegistry::mCameraEntities`, a `std::optional<EntityContext>` (20h bytes): the
/// entity registry, the entt registry, the entity id, then the engaged flag. The `CameraRegistry.cameraEntityStride`
/// entry must be the size of this mirror.
///
/// The registry function reads `[elem+8]` and `[elem+10h]` on both builds, and the push of an element sets the engaged
/// byte at `[elem+18h]`.
#[repr(C)]
pub struct CameraEntityLayout {
    _registry: *mut c_void,  // 0x00 EntityContext::mRegistry
    pub entt: *const c_void, // 0x08 EntityContext::mEnTTRegistry
    pub entity: u32,         // 0x10 EntityContext::mEntity
    _pad: u32,               // 0x14
    pub engaged: u8,         // 0x18 the optional's flag
    _tail: [u8; 7],          // 0x19
}

impl CameraEntityLayout {
    /// The size of an element, the value the `CameraRegistry.cameraEntityStride` entry must have.
    pub const STRIDE: u64 = size_of::<CameraEntityLayout>() as u64;
}

const _: () = assert!(offset_of!(CameraEntityLayout, entt) == 0x08);
const _: () = assert!(offset_of!(CameraEntityLayout, entity) == 0x10);
const _: () = assert!(offset_of!(CameraEntityLayout, engaged) == 0x18);
const _: () = assert!(size_of::<CameraEntityLayout>() == 0x20);
