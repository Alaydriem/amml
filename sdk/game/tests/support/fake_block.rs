use std::alloc::{Layout, alloc, dealloc};
use std::ffi::c_void;

use amml_msvc::{MsvcHeap, MsvcString};

use super::fake_block_type::FakeBlockType;
use super::fake_hashed_string::FakeHashedString;

/// A fake `Block`: the bytes before `mBlockType`, then the block type pointer at 68h, its place on 26.51 and
/// 26.60. The block type is owned here and its name is made through a Rust heap.
#[allow(dead_code)]
#[repr(C)]
pub struct FakeBlock {
    pad: [u8; 0x68],
    block_type: *mut c_void,
    owned: Option<Box<FakeBlockType>>,
}

impl FakeBlock {
    unsafe extern "C" fn allocate(size: usize) -> *mut u8 {
        // SAFETY: the callers ask for a non-zero size.
        unsafe { alloc(Layout::from_size_align(size, 16).unwrap()) }
    }

    unsafe extern "C" fn free(ptr: *mut u8, size: usize) {
        // SAFETY: `ptr` came from `allocate` with this size.
        unsafe { dealloc(ptr, Layout::from_size_align(size, 16).unwrap()) };
    }

    fn heap() -> MsvcHeap {
        MsvcHeap::new(Self::allocate, Self::free)
    }

    /// A block whose block type is named `name`.
    pub fn named(name: &str) -> Self {
        // SAFETY: the heap functions are valid allocate and free functions.
        let text = unsafe { MsvcString::new_in(&Self::heap(), name) }.unwrap();
        let mut block_type =
            Box::new(FakeBlockType { pad: [0; 0xE0], name: FakeHashedString { hash: 0, text, last_match: 0 } });
        let block_type_ptr: *mut FakeBlockType = &raw mut *block_type;
        Self { pad: [0; 0x68], block_type: block_type_ptr.cast(), owned: Some(block_type) }
    }

    /// A block whose block type pointer is null.
    pub fn without_type() -> Self {
        Self { pad: [0; 0x68], block_type: std::ptr::null_mut(), owned: None }
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self).cast()
    }
}

impl Drop for FakeBlock {
    fn drop(&mut self) {
        if let Some(block_type) = self.owned.as_mut() {
            // SAFETY: the string was made by `new_in` with the same heap and is not used again.
            unsafe { block_type.name.text.release(&Self::heap()) };
        }
    }
}
