use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::{AmmlAmount, AmmlContainerAction, AmmlScreenKind, AmmlSlotAt, AmmlStatus};
use amml_msvc::{MsvcHeap, MsvcString};
use amml_runtime::CollectionNames;

use crate::{
    ClientAdapter, ContainerCalls, ContainerSlotData, ContainerTransferAmount, InventoryAdapter, LoaderState,
    ObjectRead,
};

static ADAPTER: OnceLock<ContainerAdapter> = OnceLock::new();
// Whether the refusal of a move off the game thread was logged.
static THREAD_REFUSAL_LOGGED: AtomicBool = AtomicBool::new(false);

/// `ItemStackBase const& ContainerScreenController::_getVisualItemStackImpl(std::string const&, int)`: rcx the
/// controller, rdx the name, r8d the index; the reference in rax, never null.
type VisualFn = unsafe extern "C" fn(*mut c_void, *const MsvcString, i32) -> *const c_void;
/// `void _handleAutoPlace(int amount, std::string const& collection, int index)`.
type AutoPlaceFn = unsafe extern "C" fn(*mut c_void, i32, *const MsvcString, i32);
/// `void _handleTakeAll(std::string const& collection, int index)`; `_handlePlaceAll` has the same shape.
type CollectionFn = unsafe extern "C" fn(*mut c_void, *const MsvcString, i32);
/// `void _handleDropItem(std::string const& collection, int index, ItemTransferAmount)`; the 12-byte amount goes by
/// address.
type DropItemFn = unsafe extern "C" fn(*mut c_void, *const MsvcString, i32, *const ContainerTransferAmount);
/// `void _handleDropSelectedItem(ItemTransferAmount)`.
type DropSelectedFn = unsafe extern "C" fn(*mut c_void, *const ContainerTransferAmount);
/// `bool ContainerManagerController::handleSwap(SlotData const&, SlotData const&)`.
type SwapFn = unsafe extern "C" fn(*mut c_void, *const ContainerSlotData, *const ContainerSlotData) -> u8;

/// The container screen calls of the `ui` table: slot counts, the stacks of the slots and the cursor, the moves of
/// the game's own buttons and the swap of two slots.
///
/// Every call runs inside the `ScreenRender` dispatch of a container view (`ScreenAdapter::container_view` proves
/// it), on the thread that dispatch runs on, so the controller is live for the call. The adapter holds no lock while
/// it calls the game: the game may run its hover handlers inside a move. A stack the adapter gives out is lent in the
/// running UI scope (`InventoryAdapter::lend_visual`) and read there with a null player.
///
/// The collection names the game wants are built as game `std::string` values with the game heap and freed after
/// the call.
pub struct ContainerAdapter {
    calls: ContainerCalls,
}

impl ContainerAdapter {
    // The amount `_handleAutoPlace` takes for the whole stack.
    const WHOLE_STACK: i32 = 0x7FFF_FFFF;

    /// Installs the adapter when every entry of `ContainerCalls` resolved. Works once per process; false when an
    /// entry is missing or it was already installed.
    pub fn install(state: &'static LoaderState) -> bool {
        let Some(calls) = ContainerCalls::new(state) else {
            curia::info!("container calls not available", { missing: "a container screen entry" });
            return false;
        };
        ADAPTER.set(Self { calls }).is_ok()
    }

    /// Whether a move may run on this thread: only the game thread, whatever thread draws the view. `REFUSED`
    /// elsewhere, with one log line. Needs no installed adapter.
    pub(crate) fn require_game_thread() -> Result<(), AmmlStatus> {
        let on_game = LoaderState::get().is_some_and(|state| state.events().gate().on_game_thread());
        if on_game {
            return Ok(());
        }
        if !THREAD_REFUSAL_LOGGED.swap(true, Ordering::AcqRel) {
            curia::info!("ui action refused", { thread: "not game" });
        }
        Err(AmmlStatus::REFUSED)
    }

    // The stacks lent before a move are refused after it: the game may have changed the slots they point into.
    fn moved() {
        if let Some(inventory) = InventoryAdapter::get() {
            inventory.forget_lent();
        }
    }

    /// The installed adapter; `None` before `install` and when an entry is missing.
    pub fn get() -> Option<&'static ContainerAdapter> {
        ADAPTER.get()
    }

    /// The number of slots of `collection` on a screen of `kind`. `NOT_AVAILABLE` for the collection `OTHER` and for
    /// a screen kind whose container size is not known.
    pub(crate) fn slot_count(kind: AmmlScreenKind, collection: u32) -> Result<u32, AmmlStatus> {
        CollectionNames::slot_count(kind, collection).ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    // The game name of the collection of `slot` and its index, when the index is inside the collection on this
    // screen.
    fn checked(kind: AmmlScreenKind, slot: AmmlSlotAt) -> Result<(&'static str, i32), AmmlStatus> {
        let name = CollectionNames::game_name(slot.collection).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        let count = Self::slot_count(kind, slot.collection)?;
        match u32::try_from(slot.index) {
            Ok(index) if index < count => Ok((name, slot.index)),
            _ => Err(AmmlStatus::INVALID_ARGUMENT),
        }
    }

    /// The stack of `slot`, lent for the running scope; 0 for an empty slot.
    pub(crate) fn item(&self, controller: usize, kind: AmmlScreenKind, slot: AmmlSlotAt) -> Result<usize, AmmlStatus> {
        let (name, index) = Self::checked(kind, slot)?;
        self.visual(controller, name, index)
    }

    /// The stack on the cursor, lent for the running scope; 0 for an empty cursor.
    pub(crate) fn cursor(&self, controller: usize) -> Result<usize, AmmlStatus> {
        self.visual(controller, CollectionNames::CURSOR, 0)
    }

    fn visual(&self, controller: usize, name: &str, index: i32) -> Result<usize, AmmlStatus> {
        let function =
            ClientAdapter::virtual_fn(controller, Some(self.calls.visual_slot)).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `_getVisualItemStackImpl`, which takes the name and the index (`VisualFn`).
        let function = unsafe { std::mem::transmute::<usize, VisualFn>(function) };
        let stack = Self::with_name(name, |string| {
            // SAFETY: `controller` is the live controller of the running container view; the name lives for the call.
            unsafe { function(std::ptr::with_exposed_provenance_mut(controller), string, index) }
        })?;
        InventoryAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?.lend_visual(stack.expose_provenance())
    }

    /// Makes `action` as the game's own button does. `DROP_CURSOR` ignores `slot`; `AUTO_PLACE`, `TAKE_ALL` and
    /// `PLACE_ALL` ignore `amount`.
    pub(crate) fn action(
        &self,
        controller: usize,
        kind: AmmlScreenKind,
        action: AmmlContainerAction,
        slot: AmmlSlotAt,
        amount: AmmlAmount,
    ) -> Result<(), AmmlStatus> {
        let this = std::ptr::with_exposed_provenance_mut::<c_void>(controller);
        if action == AmmlContainerAction::DROP_CURSOR {
            let amount = ContainerTransferAmount::new(amount).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
            // SAFETY: the entry is the address of `_handleDropSelectedItem` (`DropSelectedFn`).
            let call = unsafe { std::mem::transmute::<usize, DropSelectedFn>(self.calls.drop_selected) };
            // SAFETY: `this` is the live controller of the running container view; the amount lives for the call.
            unsafe { call(this, &amount) };
            Self::moved();
            return Ok(());
        }
        let known = [
            AmmlContainerAction::AUTO_PLACE,
            AmmlContainerAction::TAKE_ALL,
            AmmlContainerAction::PLACE_ALL,
            AmmlContainerAction::DROP_SLOT,
        ];
        if !known.contains(&action) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let (name, index) = Self::checked(kind, slot)?;
        let amount = if action == AmmlContainerAction::DROP_SLOT { amount } else { AmmlAmount::WHOLE };
        let amount = ContainerTransferAmount::new(amount).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        let result = Self::with_name(name, |string| self.slot_action(this, action, string, index, &amount))?;
        Self::moved();
        result
    }

    // One of the four moves on the slot `index` of the collection `string`.
    fn slot_action(
        &self,
        this: *mut c_void,
        action: AmmlContainerAction,
        string: *const MsvcString,
        index: i32,
        amount: &ContainerTransferAmount,
    ) -> Result<(), AmmlStatus> {
        match action {
            AmmlContainerAction::AUTO_PLACE => {
                // SAFETY: the entry is the address of `_handleAutoPlace` (`AutoPlaceFn`).
                let call = unsafe { std::mem::transmute::<usize, AutoPlaceFn>(self.calls.auto_place) };
                // SAFETY: `this` is the live controller of the running container view; the name lives for the call.
                unsafe { call(this, Self::WHOLE_STACK, string, index) };
            }
            AmmlContainerAction::TAKE_ALL => {
                // SAFETY: the entry is the address of `_handleTakeAll` (`CollectionFn`).
                let call = unsafe { std::mem::transmute::<usize, CollectionFn>(self.calls.take_all) };
                // SAFETY: as above.
                unsafe { call(this, string, index) };
            }
            AmmlContainerAction::PLACE_ALL => {
                let function = ClientAdapter::virtual_fn(this.expose_provenance(), Some(self.calls.place_all_slot))
                    .ok_or(AmmlStatus::NOT_AVAILABLE)?;
                // SAFETY: the slot holds `_handlePlaceAll` (`CollectionFn`).
                let call = unsafe { std::mem::transmute::<usize, CollectionFn>(function) };
                // SAFETY: as above.
                unsafe { call(this, string, index) };
            }
            _ => {
                // SAFETY: the entry is the address of `_handleDropItem` (`DropItemFn`).
                let call = unsafe { std::mem::transmute::<usize, DropItemFn>(self.calls.drop_item) };
                // SAFETY: as above; the amount lives for the call.
                unsafe { call(this, string, index, amount) };
            }
        }
        Ok(())
    }

    /// Swaps the stacks of `a` and `b` through the container manager controller; whether the game made the swap.
    pub(crate) fn swap(
        &self,
        controller: usize,
        kind: AmmlScreenKind,
        a: AmmlSlotAt,
        b: AmmlSlotAt,
    ) -> Result<bool, AmmlStatus> {
        let (name_a, index_a) = Self::checked(kind, a)?;
        let (name_b, index_b) = Self::checked(kind, b)?;
        // SAFETY: `controller` is the live controller; the entry is the offset of its manager shared_ptr, whose first
        // qword is the object pointer.
        let manager =
            unsafe { ObjectRead::pointer(controller, self.calls.manager) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let function =
            ClientAdapter::virtual_fn(manager, Some(self.calls.swap_slot)).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `handleSwap` (`SwapFn`).
        let call = unsafe { std::mem::transmute::<usize, SwapFn>(function) };
        let heap = Self::heap();
        // SAFETY: `heap` is the game's operator new and delete pair.
        let mut first = unsafe { MsvcString::new_in(&heap, name_a) }.map_err(|_| AmmlStatus::FAILED)?;
        // SAFETY: as above.
        let second = match unsafe { MsvcString::new_in(&heap, name_b) } {
            Ok(string) => string,
            Err(_) => {
                // SAFETY: the string came from `new_in` with this heap and is not used again.
                unsafe { first.release(&heap) };
                return Err(AmmlStatus::FAILED);
            }
        };
        let mut slots =
            [ContainerSlotData { name: first, index: index_a }, ContainerSlotData { name: second, index: index_b }];
        // SAFETY: `manager` is the live manager of the controller; both slot data live for the call.
        let swapped = unsafe { call(std::ptr::with_exposed_provenance_mut(manager), &slots[0], &slots[1]) } != 0;
        Self::moved();
        for slot in &mut slots {
            // SAFETY: the string came from `new_in` with this heap, the call did not keep it and it is not used
            // again.
            unsafe { slot.name.release(&heap) };
        }
        Ok(swapped)
    }

    fn heap() -> MsvcHeap {
        MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete)
    }

    // Runs `call` with `name` as a game `std::string`, then frees the string.
    fn with_name<T>(name: &str, call: impl FnOnce(*const MsvcString) -> T) -> Result<T, AmmlStatus> {
        let heap = Self::heap();
        // SAFETY: `heap` is the game's operator new and delete pair.
        let mut string = unsafe { MsvcString::new_in(&heap, name) }.map_err(|_| AmmlStatus::FAILED)?;
        let result = call(&string);
        // SAFETY: the string came from `new_in` with this heap and the call did not keep it.
        unsafe { string.release(&heap) };
        Ok(result)
    }
}
