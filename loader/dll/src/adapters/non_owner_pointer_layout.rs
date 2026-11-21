use std::ffi::c_void;

use amml_msvc::SharedPtr;

/// Layout mirror of the game's `Bedrock::NonOwnerPointer<T>` (24 bytes), as `ClientInstance::getMinecraftInput`
/// writes it through its hidden result pointer: the shared pointer to the control block (whose element is the valid
/// flag byte, 1 while the object lives), then the object pointer. The copy holds one use of the control block.
#[repr(C)]
pub struct NonOwnerPointerLayout {
    pub control: SharedPtr<u8>,
    pub pointer: *mut c_void,
}

const _: () = assert!(std::mem::size_of::<NonOwnerPointerLayout>() == 24);
