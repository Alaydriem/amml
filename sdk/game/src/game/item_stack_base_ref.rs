use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{CompoundTagRef, HashedStringLayout, ItemInstanceLayout, RawObject};
use crate::{EntryId, GameEntries};

/// An `ItemStackBase*` of the game (an `ItemStack`, or the game's empty stack). The wrapper only reads and calls;
/// it never changes the stack. Client main thread; the calls are not thread-checked.
pub struct ItemStackBaseRef<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> ItemStackBaseRef<'a> {
    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `ItemStackBase` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    // Wraps a pointer the caller knows is live and non-null.
    pub(crate) fn new(ptr: NonNull<c_void>, entries: &'a GameEntries) -> Self {
        Self { ptr, entries }
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *const c_void {
        self.ptr.as_ptr()
    }

    #[allow(unsafe_code)]
    fn layout(&self) -> &ItemInstanceLayout {
        // SAFETY: `from_raw` guarantees a live ItemStackBase, which the mirror fits.
        unsafe { &*self.ptr.as_ptr().cast::<ItemInstanceLayout>() }
    }

    // The `Item` of the stack, as `WeakPtr<Item>::get` reads it; `None` for an empty stack.
    #[allow(unsafe_code)]
    fn item(&self) -> Option<*mut c_void> {
        // SAFETY: a live stack holds a null `mItem` or a live counter.
        let item = unsafe { self.layout().item() };
        (!item.is_null()).then_some(item.cast_mut())
    }

    // The function in the `Item` vtable slot that `id` names, with the item.
    #[allow(unsafe_code)]
    fn item_function(&self, id: EntryId) -> Option<(*mut c_void, *const c_void)> {
        let slot = self.entries.get(id)?;
        let item = self.item()?;
        // SAFETY: the item is live, its first qword is its vtable, and `id` is a slot entry of the Item class.
        let function = unsafe { RawObject::slot(item, slot) }?;
        Some((item, function))
    }

    // The code address that `id` names, for a non-virtual function; `None` for 0.
    fn function_address(&self, id: EntryId) -> Option<*const c_void> {
        let address = usize::try_from(self.entries.get(id)?).ok()?;
        (address != 0).then_some(address as *const c_void)
    }

    /// Whether the stack holds no item (`mItem` is null), as the game's empty stack does.
    pub fn is_empty(&self) -> bool {
        self.item().is_none()
    }

    /// The full name of the item, such as `minecraft:red_shulker_box`, from `Item::mFullName`. `None` for an empty
    /// stack, when the entry is off, or when the name is not valid UTF-8.
    #[allow(unsafe_code)]
    pub fn type_name(&self) -> Option<String> {
        let offset = usize::try_from(self.entries.get(EntryId::ITEM_FULL_NAME)?).ok()?;
        let item = self.item()?;
        // SAFETY: the item is live and `mFullName` is a HashedString at the entry's offset in it.
        let name = unsafe { &*item.cast::<u8>().add(offset).cast::<HashedStringLayout>() };
        name.text.to_str().ok().map(str::to_owned)
    }

    /// The stack size (`mCount`). `None` for an empty stack.
    pub fn count(&self) -> Option<u8> {
        self.item().map(|_| self.layout().count)
    }

    /// The user data of the stack (`mUserData`), such as the `Items` list of a shulker box. `None` when the NBT
    /// entries are off or the stack has no user data.
    #[allow(unsafe_code)]
    pub fn user_data(&self) -> Option<CompoundTagRef<'a>> {
        self.entries.get(EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX)?;
        // SAFETY: a live stack holds null or a live CompoundTag in `mUserData`.
        unsafe { CompoundTagRef::from_raw(self.layout().user_data, self.entries) }
    }

    /// Whether the stack is damaged, through `ItemStackBase::isDamaged` (not damageable, unbreakable and undamaged
    /// stacks give `false`). `None` for an empty stack or when the entry is off.
    #[allow(unsafe_code)]
    pub fn is_damaged(&self) -> Option<bool> {
        let function = self.function_address(EntryId::ITEM_STACK_BASE_IS_DAMAGED)?;
        self.item()?;
        // SAFETY: the entry is the address of `bool ItemStackBase::isDamaged() const`: this in rcx.
        let call: unsafe extern "C" fn(*const c_void) -> u8 = unsafe { std::mem::transmute(function) };
        // SAFETY: the stack is live and has an item.
        Some(unsafe { call(self.ptr.as_ptr()) } != 0)
    }

    /// The damage of the stack, through `ItemStackBase::getDamageValue`. `None` for an empty stack or when the
    /// entry is off.
    #[allow(unsafe_code)]
    pub fn damage(&self) -> Option<i32> {
        let function = self.function_address(EntryId::ITEM_STACK_BASE_GET_DAMAGE_VALUE)?;
        self.item()?;
        // SAFETY: the entry is the address of `short ItemStackBase::getDamageValue() const`: this in rcx, the
        // result in ax.
        let call: unsafe extern "C" fn(*const c_void) -> i16 = unsafe { std::mem::transmute(function) };
        // SAFETY: the stack is live and has an item.
        Some(i32::from(unsafe { call(self.ptr.as_ptr()) }))
    }

    /// The most damage the item takes, through `Item::getMaxDamage`. `None` for an empty stack or when the entry is
    /// off.
    #[allow(unsafe_code)]
    pub fn max_damage(&self) -> Option<i32> {
        let (item, function) = self.item_function(EntryId::ITEM_GET_MAX_DAMAGE)?;
        // SAFETY: the slot holds `short Item::getMaxDamage() const`: this in rcx, the result in ax.
        let call: unsafe extern "C" fn(*mut c_void) -> i16 = unsafe { std::mem::transmute(function) };
        // SAFETY: the item is live.
        Some(i32::from(unsafe { call(item) }))
    }

    /// Whether the stack shows the enchantment glint, through `Item::isGlint`. `None` for an empty stack or when
    /// the entry is off.
    #[allow(unsafe_code)]
    pub fn is_glint(&self) -> Option<bool> {
        let (item, function) = self.item_function(EntryId::ITEM_IS_GLINT)?;
        // SAFETY: the slot holds `bool Item::isGlint(ItemStackBase const&) const`: this in rcx, the stack in rdx.
        let call: unsafe extern "C" fn(*mut c_void, *const c_void) -> u8 = unsafe { std::mem::transmute(function) };
        // SAFETY: the item and the stack are live.
        Some(unsafe { call(item, self.ptr.as_ptr()) } != 0)
    }

    /// The animation frame of the item, through `Item::getAnimationFrameFor(holder, false, stack, true)`, which is
    /// the call the game's own item drawing makes. `holder` is the `Mob*` that holds the item (the local player), or
    /// null. `None` for an empty stack or when the entry is off.
    ///
    /// # Safety
    /// `holder` must be null or a live `Mob` of the game build the entries were resolved for.
    #[allow(unsafe_code)]
    pub unsafe fn animation_frame(&self, holder: *const c_void) -> Option<i32> {
        let (item, function) = self.item_function(EntryId::ITEM_GET_ANIMATION_FRAME_FOR)?;
        // SAFETY: the slot holds `int Item::getAnimationFrameFor(Mob*, bool, ItemStack const*, bool) const`: this in
        // rcx, the holder in rdx, the first bool in r8b, the stack in r9, the second bool on the stack.
        let call: unsafe extern "C" fn(*mut c_void, *const c_void, u8, *const c_void, u8) -> i32 =
            unsafe { std::mem::transmute(function) };
        // SAFETY: the item and the stack are live, and the caller guarantees the holder.
        Some(unsafe { call(item, holder, 0, self.ptr.as_ptr(), 1) })
    }
}
