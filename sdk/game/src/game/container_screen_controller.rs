use std::ffi::c_void;
use std::ptr::NonNull;

use amml_msvc::MsvcString;

use crate::game::{ItemStackBaseRef, RawObject};
use crate::{EntryId, GameEntries};

/// A `ContainerScreenController*` of the game: the controller of the inventory and chest screens. Client main
/// thread; the reads are not thread-checked.
pub struct ContainerScreenController<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> ContainerScreenController<'a> {
    /// The feature that holds the hover entries.
    pub const FEATURE: &'static str = "game.container_hover";

    /// The entries of `FEATURE`: the container test, the controller destructor, the hover and unhover functions,
    /// the visual stack slot, and the shulker box hover text functions.
    pub const ENTRIES: [EntryId; 7] = [
        EntryId::SCREEN_CONTROLLER_IS_CONTAINER_SCREEN,
        EntryId::SCREEN_CONTROLLER_DTOR,
        EntryId::CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_HOVERED,
        EntryId::CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_UNHOVERED,
        EntryId::CONTAINER_SCREEN_CONTROLLER_GET_VISUAL_ITEM_STACK_IMPL,
        EntryId::SHULKER_BOX_BLOCK_ITEM_APPEND_FORMATTED_HOVERTEXT,
        EntryId::ITEM_APPEND_FORMATTED_HOVERTEXT,
    ];

    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `ContainerScreenController` of the game build `entries` was resolved for, and
    /// must stay live while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The address of the controller, to compare with a controller seen earlier.
    pub fn address(&self) -> usize {
        self.ptr.as_ptr() as usize
    }

    /// The stack shown in slot `index` of `collection`, through `_getVisualItemStackImpl`. The game returns a
    /// reference that is never null: an empty slot gives the game's empty stack, whose `mItem` is null (see
    /// `ItemStackBaseRef::is_empty`). `None` when the entry is off or the game returns null. The stack belongs to
    /// the game and is valid until the screen changes.
    #[allow(unsafe_code)]
    pub fn visual_item(&self, collection: &MsvcString, index: i32) -> Option<ItemStackBaseRef<'a>> {
        let slot = self.entries.get(EntryId::CONTAINER_SCREEN_CONTROLLER_GET_VISUAL_ITEM_STACK_IMPL)?;
        // SAFETY: `from_raw` guarantees a live controller, whose first qword is its vtable, and the entry is the slot
        // of `_getVisualItemStackImpl` in it.
        let function = unsafe { RawObject::slot(self.ptr.as_ptr(), slot) }?;
        // SAFETY: the slot holds `ItemStackBase const& _getVisualItemStackImpl(std::string const&, int)`: this in
        // rcx, the string reference in rdx, the index in r8d, the reference returned as a pointer.
        let call: unsafe extern "C" fn(*mut c_void, *const MsvcString, i32) -> *mut c_void =
            unsafe { std::mem::transmute(function) };
        // SAFETY: the signature is as above and `collection` lives across the call.
        let stack = unsafe { call(self.ptr.as_ptr(), std::ptr::from_ref(collection), index) };
        // SAFETY: the game returns a live ItemStackBase, or null.
        unsafe { ItemStackBaseRef::from_raw(stack, self.entries) }
    }
}
