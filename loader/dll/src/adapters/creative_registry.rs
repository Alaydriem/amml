use std::ffi::c_void;

use amml_msvc::{MemberCall, RefCountBase};

use crate::{InventoryAdapter, ObjectRead, PickCalls};

/// `uchar ItemStackBase::getMaxStackSize() const`: rcx the item instance, the size in al.
type MaxStackFn = unsafe extern "C" fn(*const c_void) -> u8;

/// The creative item list of the level of a player, as `ItemRegistryRef::getCreativeItemRegistry` gives it: the
/// `ItemRegistryRef` (a weak pointer) and the `NonOwnerPointer<CreativeItemRegistry>` (a shared control block and the
/// registry) are held while the list is read, and each is given back once when this is dropped.
pub struct CreativeRegistry {
    // ItemRegistryRef: the weak_ptr<ItemRegistry> (element, control block).
    registry_ref: [usize; 2],
    // NonOwnerPointer<CreativeItemRegistry>: the shared_ptr<ControlBlock> (element, control block), then the registry.
    creative: [usize; 3],
    items_at: usize,
    max_stack: usize,
}

impl CreativeRegistry {
    /// The creative registry of the level of `player`; `None` for a null level or registry.
    ///
    /// # Safety
    /// `player` is the live local player of the build `calls` was resolved for, on the game thread.
    pub(crate) unsafe fn of_player(player: usize, calls: &PickCalls) -> Option<Self> {
        // SAFETY: the caller guarantees a live player; the entry is the offset of its level pointer.
        let level = unsafe { ObjectRead::value::<usize>(player, calls.level_at) }.filter(|level| *level != 0)?;
        let function = InventoryAdapter::virtual_fn(level, calls.item_registry_slot)?;
        // SAFETY: the slot holds `ItemRegistryRef Level::getItemRegistry() const`, a member function that returns a
        // two-word class through the hidden pointer; `level` is the live level.
        let registry_ref = unsafe {
            MemberCall::call_sret0::<[usize; 2]>(
                std::ptr::with_exposed_provenance(function),
                std::ptr::with_exposed_provenance_mut(level),
            )
        };
        // SAFETY: the entry is the address of `getCreativeItemRegistry`, whose `this` is the registry reference.
        let creative = unsafe {
            MemberCall::call_sret0::<[usize; 3]>(
                std::ptr::with_exposed_provenance(calls.creative_registry),
                registry_ref.as_ptr().cast_mut().cast(),
            )
        };
        let made = Self { registry_ref, creative, items_at: calls.creative_items_at, max_stack: calls.max_stack };
        (made.creative[2] != 0).then_some(made)
    }

    /// The addresses of the entries of the creative menu, in its order; none for a vector that is not whole entries.
    pub(crate) fn entries(&self, entry_size: usize) -> Vec<usize> {
        // SAFETY: the live registry, held for the life of this value, holds its vector (first, last) at the entry's
        // offset.
        let Some([first, last]) = (unsafe { ObjectRead::value::<[usize; 2]>(self.creative[2], self.items_at) }) else {
            return Vec::new();
        };
        let Some(bytes) = last.checked_sub(first) else { return Vec::new() };
        if first == 0 || bytes % entry_size != 0 {
            return Vec::new();
        }
        (0..bytes / entry_size).map(|index| first + index * entry_size).collect()
    }

    /// The most items one stack of the item instance at `instance` holds, as `ItemStackBase::getMaxStackSize` gives it.
    pub(crate) fn max_stack_size(&self, instance: usize) -> u8 {
        // SAFETY: the entry is the address of `getMaxStackSize`.
        let call = unsafe { std::mem::transmute::<usize, MaxStackFn>(self.max_stack) };
        // SAFETY: a live item instance of a creative entry, on the game thread.
        unsafe { call(std::ptr::with_exposed_provenance(instance)) }
    }
}

impl Drop for CreativeRegistry {
    fn drop(&mut self) {
        let control = self.creative[1] as *mut RefCountBase;
        if !control.is_null() {
            // SAFETY: the getter gave the shared_ptr with one use, which this value owns and gives back once.
            unsafe { RefCountBase::decref(control) };
        }
        let weak = self.registry_ref[1] as *mut RefCountBase;
        if !weak.is_null() {
            // SAFETY: the weak_ptr the level gave holds one weak reference, given back once.
            unsafe { RefCountBase::decwref(weak) };
        }
    }
}
