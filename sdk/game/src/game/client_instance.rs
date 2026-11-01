use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{Actor, BlockSource, RawObject};
use crate::{EntryId, GameEntries};

/// A `ClientInstance*` of the game.
pub struct ClientInstance<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> ClientInstance<'a> {
    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `ClientInstance` of the game build `entries` was resolved for, and must stay
    /// live while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    // The function in the vtable slot that `id` names, on `object`.
    #[allow(unsafe_code)]
    fn function(&self, object: *mut c_void, id: EntryId) -> Option<*const c_void> {
        let slot = self.entries.get(id)?;
        // SAFETY: the callers pass a live ClientInstance or IOptionRegistry, whose first qword is its vtable, and
        // `id` is a slot entry of that class.
        unsafe { RawObject::slot(object, slot) }
    }

    /// The local player. `None` when out of a world (the game returns null) or the entry is off. Game thread; the
    /// read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn local_player(&self) -> Option<Actor<'a>> {
        let function = self.function(self.ptr.as_ptr(), EntryId::CLIENT_INSTANCE_GET_LOCAL_PLAYER)?;
        // SAFETY: the slot holds `LocalPlayer* ClientInstance::getLocalPlayer() const`.
        let player = unsafe { RawObject::call_pointer(function, self.ptr.as_ptr()) }?;
        // SAFETY: the game returns a live LocalPlayer, which is an Actor.
        unsafe { Actor::from_raw(player, self.entries) }
    }

    /// The block source of the client's world. `None` out of a world (the game returns null) or when the entry is
    /// off. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn region(&self) -> Option<BlockSource<'a>> {
        let function = self.function(self.ptr.as_ptr(), EntryId::CLIENT_INSTANCE_GET_REGION)?;
        // SAFETY: the slot holds `BlockSource* ClientInstance::getRegion() const`.
        let source = unsafe { RawObject::call_pointer(function, self.ptr.as_ptr()) }?;
        // SAFETY: the game returns a live BlockSource.
        unsafe { BlockSource::from_raw(source, self.entries) }
    }

    /// Whether a world is running; `false` when the entry is off. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn is_world_active(&self) -> bool {
        self.function(self.ptr.as_ptr(), EntryId::CLIENT_INSTANCE_IS_WORLD_ACTIVE).is_some_and(|function| {
            // SAFETY: the slot holds `bool ClientInstance::isWorldActive() const`.
            unsafe { RawObject::call_bool(function, self.ptr.as_ptr()) }
        })
    }

    /// Whether a menu screen is showing; `false` when the entry is off. Game thread; the read is not
    /// thread-checked.
    #[allow(unsafe_code)]
    pub fn is_showing_menu(&self) -> bool {
        self.function(self.ptr.as_ptr(), EntryId::CLIENT_INSTANCE_IS_SHOWING_MENU).is_some_and(|function| {
            // SAFETY: the slot holds `bool ClientInstance::isShowingMenu() const`.
            unsafe { RawObject::call_bool(function, self.ptr.as_ptr()) }
        })
    }

    /// Whether the game holds the mouse; `false` when the entry is off. Game thread; the read is not
    /// thread-checked.
    #[allow(unsafe_code)]
    pub fn mouse_grabbed(&self) -> bool {
        self.function(self.ptr.as_ptr(), EntryId::CLIENT_INSTANCE_GET_MOUSE_GRABBED).is_some_and(|function| {
            // SAFETY: the slot holds `bool IClientInstance::getMouseGrabbed() const`.
            unsafe { RawObject::call_bool(function, self.ptr.as_ptr()) }
        })
    }

    /// Whether the HUD is hidden, through `getOptions` and then `IOptionRegistry::getHideHud`. `None` when either
    /// entry is off or the options pointer is null. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn hide_hud(&self) -> Option<bool> {
        let get_options = self.function(self.ptr.as_ptr(), EntryId::CLIENT_INSTANCE_GET_OPTIONS)?;
        self.entries.get(EntryId::I_OPTION_REGISTRY_GET_HIDE_HUD)?;
        // SAFETY: the slot holds `IOptionRegistry& ClientInstance::getOptions()`, a reference returned as a pointer.
        let options = unsafe { RawObject::call_pointer(get_options, self.ptr.as_ptr()) }?;
        let get_hide_hud = self.function(options, EntryId::I_OPTION_REGISTRY_GET_HIDE_HUD)?;
        // SAFETY: the slot holds `bool IOptionRegistry::getHideHud() const`.
        Some(unsafe { RawObject::call_bool(get_hide_hud, options) })
    }

    /// The `IMinecraftGame*` of the client, through `getMinecraftGame_DEPRECATED`. `None` when the entry is off or
    /// the game returns null. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn minecraft_game(&self) -> Option<*mut c_void> {
        let function = self.function(self.ptr.as_ptr(), EntryId::CLIENT_INSTANCE_GET_MINECRAFT_GAME)?;
        // SAFETY: the slot holds `IMinecraftGame* IClientInstance::getMinecraftGame_DEPRECATED() const`.
        unsafe { RawObject::call_pointer(function, self.ptr.as_ptr()) }
    }
}
