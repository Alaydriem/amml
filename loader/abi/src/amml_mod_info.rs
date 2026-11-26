use crate::{AmmlStr, AmmlUses};

/// What a mod reports about itself from `amml_mod_load`.
///
/// `size` is `size_of::<AmmlModInfo>()` of the mod build. `api_version` is the lowest
/// `AmmlApi::version` the mod accepts; the loader refuses a value above its own version.
///
/// `name` and `namespace` are owned by the mod and stay valid until `amml_mod_unload` returns;
/// normally they are static strings. The mod id is `@<namespace>/<name>`.
///
/// `uses` is null when `size < SIZE_V3`; read it through `AmmlModInfo::uses`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlModInfo {
    pub size: u32,
    pub api_version: u32,
    pub name: AmmlStr,
    pub namespace: AmmlStr,
    pub uses: *const AmmlUses,
}

impl AmmlModInfo {
    /// `size` of a v1 info block. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 24;

    /// `size` of a v2 info block: v1 plus `namespace`.
    pub const SIZE_V2: u32 = 40;

    /// `size` of a v3 info block: v2 plus `uses`.
    pub const SIZE_V3: u32 = 48;

    /// Reads `(size, api_version)` through a raw pointer, touching only the two leading fields.
    /// Returns `None` for a null pointer.
    ///
    /// A loader reads these first and forms an `AmmlModInfo` value only when `size` covers it.
    ///
    /// # Safety
    /// A non-null `info` must point to at least 8 readable bytes, suitably aligned.
    #[allow(unsafe_code)]
    pub unsafe fn header(info: *const AmmlModInfo) -> Option<(u32, u32)> {
        if info.is_null() {
            return None;
        }
        // SAFETY: `info` is non-null and the caller guarantees the two leading u32 fields are readable.
        let size = unsafe { core::ptr::addr_of!((*info).size).read() };
        // SAFETY: as above.
        let api_version = unsafe { core::ptr::addr_of!((*info).api_version).read() };
        Some((size, api_version))
    }

    /// Reads the `uses` pointer through a raw pointer. Gives `None` for a null block, a block
    /// shorter than `SIZE_V3` or a null `uses` field.
    ///
    /// # Safety
    /// A non-null `info` must point to at least `size` readable bytes, suitably aligned.
    #[allow(unsafe_code)]
    pub unsafe fn uses(info: *const AmmlModInfo) -> Option<*const AmmlUses> {
        if info.is_null() {
            return None;
        }
        // SAFETY: `info` is non-null and the caller guarantees the leading `size` field is readable.
        let size = unsafe { core::ptr::addr_of!((*info).size).read() };
        if size < Self::SIZE_V3 {
            return None;
        }
        // SAFETY: `size` covers the `uses` field, so it is readable.
        let uses = unsafe { core::ptr::addr_of!((*info).uses).read() };
        if uses.is_null() { None } else { Some(uses) }
    }

    /// Whether a loader speaking `loader_version` can serve this mod: the mod asks for no newer
    /// API than the loader has, and its info block holds every v2 field.
    pub fn accepted_by(&self, loader_version: u32) -> bool {
        self.api_version <= loader_version && self.size >= Self::SIZE_V2
    }
}

const _: () = assert!(core::mem::size_of::<AmmlModInfo>() == AmmlModInfo::SIZE_V3 as usize);
