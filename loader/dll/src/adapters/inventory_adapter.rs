use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use amml_abi::{AmmlHand, AmmlItemSnapshot, AmmlStatus};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::{RequestSlot, StackIdentity, StackLedger};

use crate::{ClientAdapter, InventoryLayout, ItemDrawAdapter, LoaderState, UiScope};

static ADAPTER: OnceLock<InventoryAdapter> = OnceLock::new();
// The game frames begun so far; part of the scope a lent stack is valid in.
static FRAME: AtomicU64 = AtomicU64::new(0);
// Whether the first identity of the process was logged.
static IDENTITY_LOGGED: AtomicBool = AtomicBool::new(false);
// Whether the slot stack type mismatch was logged.
static VTABLE_MISMATCH_LOGGED: AtomicBool = AtomicBool::new(false);

/// `short ItemStackBase::getDamageValue() const`: rcx the stack, the value in ax.
type DamageFn = unsafe extern "C" fn(*const c_void) -> i16;
/// `short Item::getMaxDamage() const`: rcx the item, the value in ax.
type MaxDamageFn = unsafe extern "C" fn(*const c_void) -> i16;
/// `uchar ItemStackBase::getMaxStackSize() const`: rcx the stack, the size in al (0FFh for a stack without an item).
type MaxStackFn = unsafe extern "C" fn(*const c_void) -> u8;
/// `bool Item::isGlint(ItemStackBase const&) const`: rcx the item, rdx the stack, the flag in al.
type GlintFn = unsafe extern "C" fn(*const c_void, *const c_void) -> u8;
/// `void FillingContainer::swapSlots(int, int)`: rcx the container, edx and r8d the two slots.
type SwapSlotsFn = unsafe extern "C" fn(*mut c_void, i32, i32);
/// `void Container::setItem(int, ItemStack const&)`: rcx the container, edx the slot, r8 the stack the game copies.
type SetItemFn = unsafe extern "C" fn(*mut c_void, i32, *const c_void);
/// `ItemStack const& Actor::getItemInHandSlot(HandSlot) const`: rcx the actor, dl the hand slot (1 the off hand);
/// the hand container's own slot in rax.
type ItemInHandFn = unsafe extern "C" fn(*const c_void, u8) -> *const c_void;

/// The `inventory.read` area: the held stack, the selected hotbar slot and the plain fields and name of a stack,
/// of the local player of the primary client.
///
/// Every call needs the game gate open (`NOT_READY`), the area offered (`NOT_AVAILABLE`), the game thread and a
/// player handle that is the current local player of the primary client (`REFUSED`). A stack handle must be the
/// stack the player holds now or a stack `slot_item` lent in the scope that runs now (`REFUSED` otherwise), so a
/// handle kept past its frame or its `ScreenRender` dispatch is refused, never read.
///
/// The area is offered only when the derived `ItemStack` layout values are those of the loader's mirror and
/// `HashedString.layout` names a mirror the loader has (see `InventoryLayout`).
pub struct InventoryAdapter {
    state: &'static LoaderState,
    layout: Option<InventoryLayout>,
    // Version 2: `Actor.getItemInHandSlot`, `ItemStackBase.mBlock`, `Block.mDirectData.mLightEmission`; 0 when
    // missing.
    item_in_hand: usize,
    stack_block: usize,
    light_emission: usize,
    // Version 3: `ItemStackBase.getMaxStackSize`, the `Item.isGlint` slot and the `ItemStack` vtable; 0 when missing.
    max_stack_fn: usize,
    glint_slot: usize,
    stack_vtable: usize,
    ledger: StackLedger,
}

impl InventoryAdapter {
    pub const READ: &'static str = "inventory.read";
    const AREA_VERSION: u32 = 1;
    const OFF_HAND_SLOT: u8 = 1;
    // The index byte of the net id variant is this far after its storage; 0 is the net id.
    const NET_ID_INDEX_AT: usize = 0x10;
    // The stack limit of a stack whose limit the loader cannot read.
    const DEFAULT_MAX_COUNT: u8 = 64;

    /// Offers `inventory.read` when its catalog shape resolved and the layout values match the mirror, and logs
    /// one `area adapter` or `area not available` line. Returns the row. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id == Self::READ)
            .collect();
        let value = |name: &str| state.resolve(name).ok();
        let layout = InventoryLayout::from_entries(&value);
        let refusal = layout.as_ref().err().copied();
        let slot = |name: &str| value(name).and_then(|value| usize::try_from(value).ok()).unwrap_or(0);
        let adapter = Self {
            state,
            layout: layout.ok(),
            item_in_hand: slot("Actor.getItemInHandSlot"),
            stack_block: slot("ItemStackBase.mBlock"),
            light_emission: slot("Block.mDirectData.mLightEmission"),
            max_stack_fn: slot("ItemStackBase.getMaxStackSize"),
            glint_slot: slot("Item.isGlint"),
            stack_vtable: slot("ItemStack.vtable"),
            ledger: StackLedger::new(),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the inventory adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            if row.shape.is_some()
                && let Some(reason) = refusal
            {
                row.shape = None;
                curia::info!("area not available", { area: row.id, reason: reason });
                continue;
            }
            match row.shape {
                Some(shape) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                None => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// Marks the start of a game frame: the stacks lent in the frame before are no longer valid.
    pub(crate) fn frame_began() {
        FRAME.fetch_add(1, Ordering::AcqRel);
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static InventoryAdapter> {
        ADAPTER.get()
    }

    /// The stack in `hand` of `player`; 0 for an empty hand. `INVALID_ARGUMENT` for an unknown hand.
    pub fn held_item(&self, player: *const c_void, hand: AmmlHand) -> Result<usize, AmmlStatus> {
        let (player, layout) = self.enter(player)?;
        match hand {
            AmmlHand::MAIN => Ok(Self::main_hand(player, layout)?.unwrap_or(0)),
            AmmlHand::OFF => Ok(self.off_hand(player, layout)?.unwrap_or(0)),
            _ => Err(AmmlStatus::INVALID_ARGUMENT),
        }
    }

    /// The selected hotbar slot of `player`, 0 to 8.
    pub fn selected_slot(&self, player: *const c_void) -> Result<u8, AmmlStatus> {
        let (player, layout) = self.enter(player)?;
        let slot = Self::selected(player, layout)?;
        u8::try_from(slot).map_err(|_| AmmlStatus::NOT_AVAILABLE)
    }

    /// The stack in `slot` (0 to 35) of `player`, lent for the scope that runs now; 0 for an empty slot.
    /// `INVALID_ARGUMENT` for a slot above 35.
    pub fn slot_item(&self, player: *const c_void, slot: u8) -> Result<usize, AmmlStatus> {
        let slot = usize::from(slot);
        if slot >= InventoryLayout::SLOTS {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let (player, layout) = self.enter(player)?;
        let (_, container) = Self::containers(player, layout)?;
        let vector = container.checked_add(layout.items).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the live container holds the std::vector<ItemStack> (first, last) at the entry's offset.
        let [first, last] = unsafe { std::ptr::with_exposed_provenance::<[usize; 2]>(vector).read_unaligned() };
        let stack = layout.stack_at(first, last, slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if Self::item_of(stack, layout) == 0 {
            return Ok(0);
        }
        if self.stack_vtable != 0 {
            // SAFETY: `stack` is a slot of the live item vector; its first qword is the vtable of the ItemStack.
            let vtable = unsafe { std::ptr::with_exposed_provenance::<usize>(stack).read_unaligned() };
            if vtable != self.stack_vtable {
                if !VTABLE_MISMATCH_LOGGED.swap(true, Ordering::AcqRel) {
                    curia::warn!("an inventory slot is not an ItemStack", { slot: slot, vtable: vtable });
                }
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
        }
        self.ledger.lend(Self::scope(), stack);
        Ok(stack)
    }

    /// The plain fields of `stack`, a stack `player` holds now or a lent stack of the scope that runs now.
    pub fn item_read(&self, player: *const c_void, stack: *const c_void) -> Result<AmmlItemSnapshot, AmmlStatus> {
        let (stack, layout) = self.held_stack(player, stack)?;
        Ok(self.fields_of(stack, layout))
    }

    /// The plain fields of `stack`, a live stack the loader owns or has checked.
    pub(crate) fn fields_of(&self, stack: usize, layout: &InventoryLayout) -> AmmlItemSnapshot {
        let mut snapshot = AmmlItemSnapshot::empty();
        // SAFETY: `stack` is live (held by the player or lent in this scope, checked); `count` is the offset of its mCount byte.
        snapshot.count = unsafe { std::ptr::with_exposed_provenance::<u8>(stack + layout.count).read() };
        let item = Self::item_of(stack, layout);
        if item == 0 {
            return snapshot;
        }
        if layout.damage_fn != 0 {
            // SAFETY: the entry is the address of `short ItemStackBase::getDamageValue() const`.
            let call = unsafe { std::mem::transmute::<usize, DamageFn>(layout.damage_fn) };
            // SAFETY: called on the live stack, which has an item, on the game thread.
            snapshot.damage = i32::from(unsafe { call(std::ptr::with_exposed_provenance(stack)) });
            snapshot.has_damage = 1;
        }
        if let Some(function) = Self::virtual_fn(item, layout.max_damage_slot) {
            // SAFETY: the slot holds `short Item::getMaxDamage() const`, a member function that takes only `this`.
            let call = unsafe { std::mem::transmute::<usize, MaxDamageFn>(function) };
            // SAFETY: called on its own object, the live item of the stack, on the game thread.
            snapshot.max_damage = i32::from(unsafe { call(std::ptr::with_exposed_provenance(item)) });
            snapshot.has_max_damage = 1;
        }
        // SAFETY: `stack` is live (checked); `aux` is the offset of its 16-bit mAuxValue.
        snapshot.aux = unsafe { std::ptr::with_exposed_provenance::<i16>(stack + layout.aux).read_unaligned() };
        snapshot.has_aux = 1;
        if self.max_stack_fn != 0 {
            // SAFETY: the entry is the address of `uchar ItemStackBase::getMaxStackSize() const`.
            let call = unsafe { std::mem::transmute::<usize, MaxStackFn>(self.max_stack_fn) };
            // SAFETY: called on the live stack, which has an item, on the game thread.
            snapshot.max_count = unsafe { call(std::ptr::with_exposed_provenance(stack)) };
            snapshot.has_max_count = 1;
        }
        if self.glint_slot != 0
            && let Some(function) = Self::virtual_fn(item, self.glint_slot)
        {
            // SAFETY: the slot holds `bool Item::isGlint(ItemStackBase const&) const`: this and the stack.
            let call = unsafe { std::mem::transmute::<usize, GlintFn>(function) };
            // SAFETY: called on the item of the live stack with that stack, on the game thread.
            let glint =
                unsafe { call(std::ptr::with_exposed_provenance(item), std::ptr::with_exposed_provenance(stack)) };
            snapshot.is_glint = u8::from(glint != 0);
            snapshot.has_glint = 1;
        }
        if let Some(identity) = Self::identity(stack, layout, snapshot.aux, snapshot.count) {
            snapshot.identity = identity;
            snapshot.has_identity = 1;
            if !IDENTITY_LOGGED.swap(true, Ordering::AcqRel) {
                curia::info!("first stack identity", { identity: identity });
            }
        }
        snapshot
    }

    // The identity of a stack; `None` for a stack with user data (its bytes need `CompoundTag.write` and an
    // output object, which the loader does not have) and for a name that cannot be read.
    fn identity(stack: usize, layout: &InventoryLayout, aux: i16, count: u8) -> Option<u64> {
        // SAFETY: `stack` is live; `user_data` is the offset of its unique_ptr<CompoundTag>.
        if unsafe { Self::pointer(stack, layout.user_data) }.is_some() {
            return None;
        }
        let item = Self::item_of(stack, layout);
        let name = item.checked_add(layout.full_name).filter(|_| item != 0)?;
        // SAFETY: the item is live and `mFullName` is a HashedString of the layout `names` was made for.
        let type_name = unsafe { layout.names.read(name) }?;
        Some(StackIdentity { type_name: &type_name, aux, count, user_data: &[] }.value())
    }

    /// The change stamp of `stack`, a stack `player` holds now or a lent stack of this scope: 0 for a stack with no
    /// user data, else the stamp of its user data tag (`TagReader::stamp`). `REFUSED` off the game thread, because the
    /// walk reads a tag the game thread may change; `NOT_AVAILABLE` without a tag reader or for a walk that fails.
    pub fn item_change_stamp(&self, player: *const c_void, stack: *const c_void) -> Result<u64, AmmlStatus> {
        if !self.state.events().gate().on_game_thread() {
            return Err(AmmlStatus::REFUSED);
        }
        let (stack, layout) = self.held_stack(player, stack)?;
        let user_data = Self::user_data_of(stack, layout);
        if user_data == 0 {
            return Ok(0);
        }
        let reader = ItemDrawAdapter::with(None, ItemDrawAdapter::tag_reader).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: `user_data` is the live CompoundTag of a stack the loader checked, read on the game thread that owns it.
        unsafe { reader.stamp(user_data) }.ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    /// The type name of `stack`, a stack `player` holds now or a lent stack of this scope; `NOT_AVAILABLE` for a name that cannot be read.
    pub fn item_name(&self, player: *const c_void, stack: *const c_void) -> Result<String, AmmlStatus> {
        let (stack, layout) = self.held_stack(player, stack)?;
        Self::name_of(stack, layout).ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    /// The type name of `stack`, a live stack the loader owns or has checked; `None` for a name that cannot be read.
    pub(crate) fn name_of(stack: usize, layout: &InventoryLayout) -> Option<String> {
        let item = Self::item_of(stack, layout);
        let name = item.checked_add(layout.full_name).filter(|_| item != 0)?;
        // SAFETY: the item is live and `mFullName` is a HashedString of the layout `names` was made for at the
        // entry's offset in it.
        unsafe { layout.names.read(name) }
    }

    /// The 36 slots of `player` as a request needs them: the net id, count, stack limit and identity of each stack, `None`
    /// for an empty slot. `None` as a whole when the item vector is not 36 stacks, or a stack holds a request id in place
    /// of a net id (a move of it still waits for the server's answer).
    pub(crate) fn request_slots(&self, player: usize, net_id_at: usize) -> Option<Vec<Option<RequestSlot>>> {
        let layout = self.layout.as_ref()?;
        let (_, first) = Self::player_items(player, layout)?;
        (0..InventoryLayout::SLOTS)
            .map(|slot| {
                let stack = first + slot * layout.stack_size;
                if Self::item_of(stack, layout) == 0 {
                    return Some(None);
                }
                let fields = self.fields_of(stack, layout);
                Some(Some(RequestSlot {
                    net_id: Self::net_id_of(stack, net_id_at)?,
                    count: fields.count,
                    max: if fields.has_max_count != 0 { fields.max_count } else { Self::DEFAULT_MAX_COUNT },
                    key: if fields.has_identity != 0 { fields.identity } else { 0 },
                }))
            })
            .collect()
    }

    /// Swaps the stacks of player slots `a` and `b` through the container's own `swapSlots` (the slot `swap_slots` of its
    /// vtable), which also tells the container's listeners. With `expect_at_b` the swap runs only while slot `b` holds the
    /// stack with that net id (`net_id_at` is the offset of the net id in a stack). False when it did not run. A swap that ran
    /// forgets the stacks lent in the scope that runs now.
    pub fn swap_player_slots(
        &self,
        player: usize,
        swap_slots: usize,
        a: u8,
        b: u8,
        expect_at_b: Option<(i32, usize)>,
    ) -> bool {
        let Some(layout) = self.layout.as_ref() else { return false };
        let Some((container, first)) = Self::player_items(player, layout) else { return false };
        if usize::from(a) >= InventoryLayout::SLOTS || usize::from(b) >= InventoryLayout::SLOTS {
            return false;
        }
        if let Some((net_id, net_id_at)) = expect_at_b {
            let stack = first + usize::from(b) * layout.stack_size;
            if Self::item_of(stack, layout) == 0 || Self::net_id_of(stack, net_id_at) != Some(net_id) {
                return false;
            }
        }
        let Some(function) = Self::virtual_fn(container, swap_slots) else { return false };
        // SAFETY: the slot holds `void FillingContainer::swapSlots(int, int)`.
        let swap = unsafe { std::mem::transmute::<usize, SwapSlotsFn>(function) };
        // SAFETY: the live container of the local player and two slots inside it, on the game thread.
        unsafe { swap(std::ptr::with_exposed_provenance_mut(container), i32::from(a), i32::from(b)) };
        // The slots changed under every handle lent before the swap: they are refused from here on.
        self.forget_lent();
        true
    }

    /// Puts a copy of the stack at `stack` into player slot `slot` through the container's own `setItem` (the slot
    /// `set_item` of its vtable), which also tells the container's listeners. False when it did not run. A write forgets
    /// the stacks lent in the scope that runs now.
    pub(crate) fn set_player_slot(&self, player: usize, set_item: usize, slot: u8, stack: usize) -> bool {
        let Some(layout) = self.layout.as_ref() else { return false };
        let Some((container, _)) = Self::player_items(player, layout) else { return false };
        if usize::from(slot) >= InventoryLayout::SLOTS || stack == 0 {
            return false;
        }
        let Some(function) = Self::virtual_fn(container, set_item) else { return false };
        // SAFETY: the slot holds `void Container::setItem(int, ItemStack const&)`.
        let set = unsafe { std::mem::transmute::<usize, SetItemFn>(function) };
        // SAFETY: the live container of the local player, a slot inside it, and a live stack the game copies, on the
        // game thread.
        unsafe {
            set(
                std::ptr::with_exposed_provenance_mut(container),
                i32::from(slot),
                std::ptr::with_exposed_provenance(stack),
            )
        };
        self.forget_lent();
        true
    }

    /// The stack in player slot `slot` while it holds an item; `None` for an empty slot.
    pub(crate) fn held_in_slot(&self, player: usize, slot: u8) -> Option<usize> {
        let layout = self.layout.as_ref()?;
        let (_, first) = Self::player_items(player, layout)?;
        if usize::from(slot) >= InventoryLayout::SLOTS {
            return None;
        }
        let stack = first + usize::from(slot) * layout.stack_size;
        (Self::item_of(stack, layout) != 0).then_some(stack)
    }

    /// The stack in player slot `slot` while it holds an item and its net id is 0 (the stack a pick predicted: the
    /// server names the real id later); `None` for another slot content or a net id that is a request id.
    pub(crate) fn unnamed_stack(&self, player: usize, slot: u8, net_id_at: usize) -> Option<usize> {
        let layout = self.layout.as_ref()?;
        let (_, first) = Self::player_items(player, layout)?;
        if usize::from(slot) >= InventoryLayout::SLOTS {
            return None;
        }
        let stack = first + usize::from(slot) * layout.stack_size;
        (Self::item_of(stack, layout) != 0 && Self::net_id_of(stack, net_id_at) == Some(0)).then_some(stack)
    }

    /// Writes `net_id` as the net id of the stack in player slot `slot`, only while `unnamed_stack` holds for it. False
    /// when it did not run.
    pub(crate) fn name_stack(&self, player: usize, slot: u8, net_id_at: usize, net_id: i32) -> bool {
        let Some(stack) = self.unnamed_stack(player, slot, net_id_at) else { return false };
        // SAFETY: a stack of the live item vector; the net id lies inside the 0x98-byte stack and the variant index byte
        // after it is 0 (the net id alternative), as `unnamed_stack` read.
        unsafe { std::ptr::with_exposed_provenance_mut::<i32>(stack + net_id_at).write_unaligned(net_id) };
        true
    }

    // The container of the player's inventory and its first stack, when the item vector holds exactly 36 whole stacks.
    fn player_items(player: usize, layout: &InventoryLayout) -> Option<(usize, usize)> {
        let (_, container) = Self::containers(player, layout).ok()?;
        let vector = container.checked_add(layout.items)?;
        // SAFETY: the live container holds the std::vector<ItemStack> (first, last) at the entry's offset.
        let [first, last] = unsafe { std::ptr::with_exposed_provenance::<[usize; 2]>(vector).read_unaligned() };
        let bytes = last.checked_sub(first)?;
        (first != 0 && bytes % layout.stack_size == 0 && bytes / layout.stack_size == InventoryLayout::SLOTS)
            .then_some((container, first))
    }

    // The net id of `stack`; `None` when its net id variant holds a request id. `net_id_at` is the offset of the variant
    // in the stack: the 4-byte id at that offset, the variant index byte `NET_ID_INDEX_AT` later (0 is the net id).
    fn net_id_of(stack: usize, net_id_at: usize) -> Option<i32> {
        // SAFETY: `stack` is a stack of the live item vector; the variant lies inside the 0x98-byte stack.
        let index =
            unsafe { std::ptr::with_exposed_provenance::<u8>(stack + net_id_at + Self::NET_ID_INDEX_AT).read() };
        if index != 0 {
            return None;
        }
        // SAFETY: as above.
        Some(unsafe { std::ptr::with_exposed_provenance::<i32>(stack + net_id_at).read_unaligned() })
    }

    /// The `CompoundTag` of the user data of `stack`, a live stack the loader owns or has checked; 0 for none.
    pub(crate) fn user_data_of(stack: usize, layout: &InventoryLayout) -> usize {
        // SAFETY: `stack` is live; `user_data` is the offset of its unique_ptr<CompoundTag>.
        unsafe { Self::pointer(stack, layout.user_data) }.unwrap_or(0)
    }

    /// The layout of the inventory mirror; `None` when the area was refused.
    pub(crate) fn layout(&self) -> Option<&InventoryLayout> {
        self.layout.as_ref()
    }

    /// `stack`, when it is a stack `player` holds now or a stack lent in the scope that runs now, with the layout.
    pub(crate) fn checked_stack(
        &self,
        player: *const c_void,
        stack: *const c_void,
    ) -> Result<(usize, &InventoryLayout), AmmlStatus> {
        self.held_stack(player, stack)
    }

    /// The block `stack` places and its light emission (0 to 15); address 0 and emission 0 for an item that places
    /// no block.
    pub fn item_block(&self, player: *const c_void, stack: *const c_void) -> Result<(usize, u8), AmmlStatus> {
        let (stack, _) = self.held_stack(player, stack)?;
        if self.stack_block == 0 || self.light_emission == 0 {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        // SAFETY: `stack` is live (held by the player or lent in this scope, checked); it holds its `Block const*` (null for an
        // item without a block) at the entry's offset.
        let Some(block) = (unsafe { Self::pointer(stack, self.stack_block) }) else { return Ok((0, 0)) };
        let address = block.checked_add(self.light_emission).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: a live Block holds its light emission byte at the entry's offset.
        let emission = unsafe { std::ptr::with_exposed_provenance::<u8>(address).read() };
        Ok((block, emission.min(15)))
    }

    // The stack in the off hand; `None` for an empty off hand.
    fn off_hand(&self, player: usize, layout: &InventoryLayout) -> Result<Option<usize>, AmmlStatus> {
        if self.item_in_hand == 0 {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        // SAFETY: the entry is the address of `ItemStack const& Actor::getItemInHandSlot(HandSlot) const`.
        let call = unsafe { std::mem::transmute::<usize, ItemInHandFn>(self.item_in_hand) };
        // SAFETY: called on the live local player on the game thread; the result is the hand container's own slot.
        let stack = unsafe { call(std::ptr::with_exposed_provenance(player), Self::OFF_HAND_SLOT) }.expose_provenance();
        Ok((stack != 0 && Self::item_of(stack, layout) != 0).then_some(stack))
    }

    // `stack` when it is the stack `player` holds in the main or the off hand now, or a stack a loader call lent in
    // the scope that runs now. A null `player` names no player: the stack must be one lent in a running UI scope of
    // this thread (a stack of a container screen), and the game thread is not required.
    fn held_stack(&self, player: *const c_void, stack: *const c_void) -> Result<(usize, &InventoryLayout), AmmlStatus> {
        if player.is_null() {
            return self.lent_stack(stack);
        }
        let (player, layout) = self.enter(player)?;
        let stack = stack.expose_provenance();
        if stack == 0 {
            return Err(AmmlStatus::REFUSED);
        }
        let main = Self::main_hand(player, layout)?;
        let off = self.off_hand(player, layout).ok().flatten();
        let held: Vec<usize> = main.into_iter().chain(off).collect();
        if self.ledger.accepts(Self::scope(), stack, &held) { Ok((stack, layout)) } else { Err(AmmlStatus::REFUSED) }
    }

    // `stack` when it was lent in the UI scope that runs now on this thread.
    fn lent_stack(&self, stack: *const c_void) -> Result<(usize, &InventoryLayout), AmmlStatus> {
        self.state.events().gate().require_open()?;
        if !self.state.areas().is_available(Self::READ, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let layout = self.layout.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let stack = stack.expose_provenance();
        if UiScope::serial() != 0 && self.ledger.accepts(Self::scope(), stack, &[]) {
            Ok((stack, layout))
        } else {
            Err(AmmlStatus::REFUSED)
        }
    }

    /// Forgets the stacks lent in the scope that runs now: a handle lent before a container move is refused after it.
    pub(crate) fn forget_lent(&self) {
        self.ledger.forget(Self::scope());
    }

    /// The game frames begun so far.
    pub(crate) fn frame() -> u64 {
        FRAME.load(Ordering::Acquire)
    }

    /// Lends the stack of a container screen slot for the UI scope that runs now: the stack itself, or 0 for the
    /// game's empty stack. Any `inventory` call with a null player reads it in this scope and no other.
    /// `NOT_AVAILABLE` without the layout; `REFUSED` outside a UI scope.
    pub fn lend_visual(&self, stack: usize) -> Result<usize, AmmlStatus> {
        let layout = self.layout.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if UiScope::serial() == 0 {
            return Err(AmmlStatus::REFUSED);
        }
        if stack == 0 || Self::item_of(stack, layout) == 0 {
            return Ok(0);
        }
        self.ledger.lend(Self::scope(), stack);
        Ok(stack)
    }

    // The scope a lent stack is valid in: the game frame and the UI dispatch that runs now (0 outside one).
    fn scope() -> u64 {
        (FRAME.load(Ordering::Acquire) << 32) | (UiScope::serial() & 0xFFFF_FFFF)
    }

    // The address of `player` and the layout when an inventory call may use it.
    fn enter(&self, player: *const c_void) -> Result<(usize, &InventoryLayout), AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(Self::READ, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let layout = self.layout.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if !gate.on_game_thread() {
            return Err(AmmlStatus::REFUSED);
        }
        let player = player.expose_provenance();
        let current = ClientAdapter::get().and_then(ClientAdapter::primary_local_player);
        match current {
            Some(current) if current == player => Ok((player, layout)),
            _ => Err(AmmlStatus::REFUSED),
        }
    }

    // The player's PlayerInventory and its container (the `Inventory`, a FillingContainer).
    fn containers(player: usize, layout: &InventoryLayout) -> Result<(usize, usize), AmmlStatus> {
        // SAFETY: `player` is the live local player; the entry is the offset of its PlayerInventory pointer.
        let supplies = unsafe { Self::pointer(player, layout.player_inventory) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: a live PlayerInventory holds its Inventory pointer at the entry's offset.
        let container = unsafe { Self::pointer(supplies, layout.container) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        Ok((supplies, container))
    }

    fn selected(player: usize, layout: &InventoryLayout) -> Result<usize, AmmlStatus> {
        let (supplies, _) = Self::containers(player, layout)?;
        let address = supplies.checked_add(layout.selected).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: a live PlayerInventory holds `int mSelected` at the entry's offset.
        let slot = unsafe { std::ptr::with_exposed_provenance::<i32>(address).read_unaligned() };
        usize::try_from(slot).ok().filter(|slot| *slot < InventoryLayout::HOTBAR).ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    // The stack in the selected slot; `None` for an empty hand (no stack there, or a stack with no item).
    fn main_hand(player: usize, layout: &InventoryLayout) -> Result<Option<usize>, AmmlStatus> {
        let slot = Self::selected(player, layout)?;
        let (_, container) = Self::containers(player, layout)?;
        let vector = container.checked_add(layout.items).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the live container holds the std::vector<ItemStack> (first, last) at the entry's offset.
        let [first, last] = unsafe { std::ptr::with_exposed_provenance::<[usize; 2]>(vector).read_unaligned() };
        Ok(layout.stack_at(first, last, slot).filter(|stack| Self::item_of(*stack, layout) != 0))
    }

    /// The Item of a stack, as `WeakPtr<Item>::get` reads it; 0 for none.
    pub(crate) fn item_of(stack: usize, layout: &InventoryLayout) -> usize {
        // SAFETY: `stack` is a live stack inside the player's item vector; mItem is null or a pointer to a
        // SharedCounter whose first member is the Item pointer.
        unsafe { Self::pointer(stack, layout.item).and_then(|counter| Self::pointer(counter, 0)) }.unwrap_or(0)
    }

    // The non-null pointer at `object + offset`.
    //
    // Safety: `object + offset` is readable for 8 bytes.
    unsafe fn pointer(object: usize, offset: usize) -> Option<usize> {
        let address = object.checked_add(offset)?;
        // SAFETY: the caller guarantees the qword is readable.
        let value = unsafe { std::ptr::with_exposed_provenance::<usize>(address).read_unaligned() };
        (value != 0).then_some(value)
    }

    // The function in `slot` of the vtable at the start of `object`; `None` for a null vtable or function.
    pub(crate) fn virtual_fn(object: usize, slot: usize) -> Option<usize> {
        // SAFETY: `object` is a live, non-null game object; its first qword is its vtable pointer.
        let vtable = unsafe { std::ptr::with_exposed_provenance::<*const usize>(object).read() };
        if vtable.is_null() {
            return None;
        }
        // SAFETY: `vtable` is the non-null vtable of the object; the slot entry's range check keeps `slot` inside it.
        let function = unsafe { vtable.add(slot).read() };
        (function != 0).then_some(function)
    }
}
