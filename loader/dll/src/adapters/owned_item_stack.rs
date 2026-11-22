use std::ffi::c_void;

use crate::{InventoryAdapter, PickCalls};

// The words of the 0x98-byte `ItemStack`.
const STACK_WORDS: usize = 0x98 / 8;

/// `ItemStack::ItemStack()` (or `ItemStackBase::ItemStackBase()`): rcx the new stack, returned in rax.
type CtorFn = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
/// `void ItemStack::reinit(Item const&, int count, int auxValue)`: this, the item, the count, the aux value.
type ReinitFn = unsafe extern "C" fn(*mut c_void, *const c_void, i32, i32);
/// `ItemStack::ItemStack(ItemStack const&)`: rcx the new stack, rdx the source, the new stack back in rax.
type CopyCtorFn = unsafe extern "C" fn(*mut c_void, *const c_void) -> *mut c_void;
/// The scalar deleting destructor in slot 0 of the `ItemStack` vtable: rcx the stack, edx the flags (bit 0 frees).
type DeletingDtorFn = unsafe extern "C" fn(*mut c_void, u32) -> *mut c_void;

/// An `ItemStack` the loader owns, in its own memory, that the game's constructor made. It is destroyed once, when it
/// is dropped, through the full `ItemStack` destructor (the vtable's scalar deleting destructor with no free).
pub struct OwnedItemStack {
    // 0x98 bytes, aligned for the stack.
    stack: Box<[u64; STACK_WORDS]>,
    vtable: usize,
}

impl OwnedItemStack {
    // The net id variant: its 32-bit value at +80h and its index byte at +90h (`mov dword ptr [x+80h],0` and
    // `mov byte ptr [x+90h],0` after the constructor on both gate builds), the empty net id.
    const NET_ID_VALUE: usize = 0x80;
    const NET_ID_INDEX: usize = 0x90;
    const KEEP_MEMORY: u32 = 0;

    /// An empty stack, as `ItemStack()` makes it: the game's empty stack constructor, then the three stores the
    /// callers of `ItemStackBase::ItemStackBase()` write inline (the `ItemStack` vtable at +0, a zero net id value and
    /// a zero variant index). `ItemStack.emptyCtor` is the whole constructor on 70.26, so there the stores write what
    /// is already there.
    ///
    /// # Safety
    /// `calls` is resolved for the running build, on the game thread.
    pub(crate) unsafe fn new(calls: &PickCalls) -> Self {
        let mut stack = Box::new([0u64; STACK_WORDS]);
        // SAFETY: the entry is the address of the empty stack constructor.
        let ctor = unsafe { std::mem::transmute::<usize, CtorFn>(calls.empty_ctor) };
        // SAFETY: 0x98 bytes of the loader's memory, aligned for the stack.
        unsafe { ctor(stack.as_mut_ptr().cast()) };
        let bytes = stack.as_mut_ptr().cast::<u8>();
        // SAFETY: inside the 0x98-byte stack: the vtable pointer at +0 (the entry is the `ItemStack` vtable) and the
        // value and index of the net id variant.
        unsafe {
            bytes.cast::<usize>().write(calls.stack_vtable);
            bytes.add(Self::NET_ID_VALUE).cast::<u32>().write_unaligned(0);
            bytes.add(Self::NET_ID_INDEX).write(0);
        }
        Self { stack, vtable: calls.stack_vtable }
    }

    /// A copy of the stack at `source`, as `ItemStack(ItemStack const&)` makes it.
    ///
    /// # Safety
    /// `source` is a live `ItemStack` of the running build, `calls` is resolved for it, on the game thread.
    pub(crate) unsafe fn copy_of(source: usize, calls: &PickCalls) -> Self {
        let mut stack = Box::new([0u64; STACK_WORDS]);
        // SAFETY: the entry is the address of the copy constructor.
        let copy = unsafe { std::mem::transmute::<usize, CopyCtorFn>(calls.copy_ctor) };
        // SAFETY: 0x98 bytes of the loader's memory, aligned for the stack, and a live source stack.
        unsafe { copy(stack.as_mut_ptr().cast(), std::ptr::with_exposed_provenance(source)) };
        Self { stack, vtable: calls.stack_vtable }
    }

    /// Makes the stack `count` items of `item` with `aux`, through the vtable's `reinit(Item const&, int, int)` at
    /// the slot `calls.reinit_slot`. False when the slot is empty.
    ///
    /// # Safety
    /// `item` is a live `Item`, and `calls` the calls this stack was made with.
    pub(crate) unsafe fn reinit(&mut self, item: usize, count: i32, aux: i32, calls: &PickCalls) -> bool {
        let this = self.stack.as_mut_ptr().cast::<c_void>();
        let Some(function) = InventoryAdapter::virtual_fn(this.expose_provenance(), calls.reinit_slot) else {
            return false;
        };
        // SAFETY: the slot holds the `Item` overload of `reinit`.
        let reinit = unsafe { std::mem::transmute::<usize, ReinitFn>(function) };
        // SAFETY: the live stack the game's constructor made, and the caller's live item.
        unsafe { reinit(this, std::ptr::with_exposed_provenance(item), count, aux) };
        true
    }

    /// The address of the stack, as the game's `setItem` takes it.
    pub(crate) fn address(&self) -> usize {
        self.stack.as_ptr().expose_provenance()
    }
}

impl Drop for OwnedItemStack {
    fn drop(&mut self) {
        // SAFETY: the entry is the `ItemStack` vtable, whose first slot holds the scalar deleting destructor.
        let function = unsafe { std::ptr::with_exposed_provenance::<usize>(self.vtable).read() };
        if function == 0 {
            return;
        }
        // SAFETY: the slot holds the scalar deleting destructor.
        let dtor = unsafe { std::mem::transmute::<usize, DeletingDtorFn>(function) };
        // SAFETY: the stack the game's constructor made, destroyed once; with no free flag the box keeps its memory.
        unsafe { dtor(self.stack.as_mut_ptr().cast(), Self::KEEP_MEMORY) };
    }
}
