use crate::AmmlStr;

/// What a mod reports about itself from `amml_mod_load`.
///
/// `size` is `size_of::<AmmlModInfo>()` of the mod build. `api_version` is the lowest
/// `AmmlApi::version` the mod accepts; the loader refuses a value above its own version.
///
/// `name` is owned by the mod and stays valid until `amml_mod_unload` returns; normally it is a
/// static string.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlModInfo {
    pub size: u32,
    pub api_version: u32,
    pub name: AmmlStr,
}

impl AmmlModInfo {
    /// `size` of a v1 info block. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 24;

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

    /// Whether a loader speaking `loader_version` can serve this mod: the mod asks for no newer
    /// API than the loader has, and its info block holds every v1 field.
    pub fn accepted_by(&self, loader_version: u32) -> bool {
        self.api_version <= loader_version && self.size >= Self::SIZE_V1
    }
}

const _: () = assert!(core::mem::size_of::<AmmlModInfo>() == AmmlModInfo::SIZE_V1 as usize);
