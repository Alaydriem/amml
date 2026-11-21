use std::ffi::c_void;

use amml_msvc::SharedPtr;

/// Mirror of `mce::TexturePtr` (20h bytes): a shared `BedrockTextureData`, then a shared `ResourceLocation`. Both are
/// owned references, so a value the game filled is released with `release`.
#[repr(C)]
pub struct TexturePtrLayout {
    pub texture: SharedPtr<c_void>,
    pub location: SharedPtr<c_void>,
}

const _: () = assert!(size_of::<TexturePtrLayout>() == 0x20);

impl TexturePtrLayout {
    /// Two empty shared pointers, ready for the game to fill.
    pub fn empty() -> Self {
        // SAFETY: an all-zero `shared_ptr` is the empty pointer.
        unsafe { std::mem::zeroed() }
    }

    /// The address of the `BedrockTextureData`; 0 when the texture pointer is empty.
    pub fn data(&self) -> usize {
        self.texture.get().map_or(0, |data| std::ptr::from_ref(data).addr())
    }

    /// Drops the use each shared pointer holds, location first, as the game destructor does.
    ///
    /// # Safety
    /// Each pointer is empty or holds a live use, and this runs at most once per filled value.
    pub unsafe fn release(&mut self) {
        // SAFETY: the caller guarantees both pointers are empty or live.
        unsafe {
            self.location.release();
            self.texture.release();
        }
    }
}
