use std::ffi::c_void;
use std::ptr::NonNull;

use amml_msvc::{MsvcHeap, ScalarDeletingDestructor};

use crate::game::{CompoundTagRef, ItemStackBaseRef, ItemStackLayout, RawObject};
use crate::{EntryId, GameEntries, GameHeap};

/// An `ItemStack` that the mod owns, made from an item tag by `ItemStack::fromTag`. The stack lives in a block of
/// the game heap; dropping it runs the game's destructor (without the free flag) and returns the block. Client
/// main thread: `fromTag` looks the item up in the game's registry.
pub struct OwnedItemStack<'a> {
    ptr: NonNull<c_void>,
    heap: MsvcHeap,
    entries: &'a GameEntries,
}

impl<'a> OwnedItemStack<'a> {
    /// Builds the stack of `tag` in a block of the game heap. `None` when the game heap is not there (no mod
    /// runtime), the `ItemStack.fromTag` entry is off, the allocation fails, or the stack is null (an unknown item).
    pub fn from_tag(tag: CompoundTagRef<'a>) -> Option<Self> {
        Self::from_tag_in(tag, GameHeap::msvc()?)
    }

    /// As [`OwnedItemStack::from_tag`], with the block taken from `heap`, which must be the game's operator new and
    /// delete.
    #[allow(unsafe_code)]
    pub fn from_tag_in(tag: CompoundTagRef<'a>, heap: MsvcHeap) -> Option<Self> {
        let entries = tag.entries();
        let function = usize::try_from(entries.get(EntryId::ITEM_STACK_FROM_TAG)?).ok()?;
        if function == 0 {
            return None;
        }
        let size = size_of::<ItemStackLayout>();
        // SAFETY: the caller guarantees the heap is the game's operator new and delete.
        let block = unsafe { heap.allocate(size) };
        let ptr = NonNull::new(block.cast::<c_void>())?;
        // SAFETY: the block is `size` writable bytes.
        unsafe { std::ptr::write_bytes(block, 0, size) };
        // SAFETY: the entry is the address of `static ItemStack ItemStack::fromTag(CompoundTag const&)`: the hidden
        // return buffer in rcx, the tag in rdx, the buffer returned in rax.
        let from_tag: unsafe extern "C" fn(*mut c_void, *const c_void) -> *mut c_void =
            unsafe { std::mem::transmute(function as *const c_void) };
        // SAFETY: the buffer is an ItemStack-sized block of the game heap, and the tag is live.
        unsafe { from_tag(ptr.as_ptr(), tag.as_ptr()) };
        // from here a drop destroys the stack and returns the block, also for the null stack
        let stack = Self { ptr, heap, entries };
        if stack.as_base().is_empty() {
            return None;
        }
        Some(stack)
    }

    /// The stack as an `ItemStackBase`, for the item reads and the item drawing. It is valid while `self` lives.
    pub fn as_base(&self) -> ItemStackBaseRef<'_> {
        ItemStackBaseRef::new(self.ptr, self.entries)
    }
}

impl Drop for OwnedItemStack<'_> {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: the stack was built by `fromTag`, so its first qword is the ItemStack vtable (or null, for a stack
        // that was never filled).
        if let Some(function) = unsafe { RawObject::slot(self.ptr.as_ptr(), 0) } {
            // SAFETY: slot 0 of an ItemStack is its scalar deleting destructor; flags 0 destroys without freeing.
            let destroy: ScalarDeletingDestructor = unsafe { std::mem::transmute(function) };
            // SAFETY: the stack is live and destroyed exactly once.
            unsafe { destroy(self.ptr.as_ptr(), 0) };
        }
        // SAFETY: the block came from this heap with this size.
        unsafe { self.heap.deallocate(self.ptr.as_ptr().cast::<u8>(), size_of::<ItemStackLayout>()) };
    }
}
