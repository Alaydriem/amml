use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{AmmlContainerAction, AmmlInventoryTable, AmmlScreenKind, AmmlSlotAt, AmmlUiTable};

use crate::{Amount, ApiError, Collection, ItemSnapshot, ItemView, ScreenId, ScreenKind, SlotAt, ViewId};

/// The container screen of a `ScreenRender` view (a chest, a barrel, the player inventory and the like), as
/// `ScreenRenderExt::container_screen` gives it, for the call of the listener that got the event.
///
/// It reads the slots and the cursor, and makes the moves of the game's own buttons: the game predicts the move, sends
/// its request and corrects the screen itself. Every call goes through the loader (`Area::UI_SCREEN.at(2)`), which
/// checks that the call of the event still runs on this thread; a read it refuses gives `None` and a move it refuses
/// gives an error. A slot index outside its collection is `ApiError::Loader` with `INVALID_ARGUMENT`.
pub struct ContainerScreen<'f> {
    table: AmmlUiTable,
    inventory: Option<AmmlInventoryTable>,
    module: &'static str,
    view: ViewId,
    screen: ScreenId,
    _event: PhantomData<&'f ()>,
}

impl<'f> ContainerScreen<'f> {
    pub(crate) fn new(
        table: AmmlUiTable,
        inventory: Option<AmmlInventoryTable>,
        module: &'static str,
        view: ViewId,
        screen: ScreenId,
    ) -> Self {
        Self { table, inventory, module, view, screen, _event: PhantomData }
    }

    /// The id of the screen, the id of `ScreenRender`.
    pub fn id(&self) -> ScreenId {
        self.screen
    }

    /// The kind of the screen; `Other` when it cannot be read.
    pub fn kind(&self) -> ScreenKind {
        let mut out = AmmlScreenKind::OTHER;
        // SAFETY: the loader checks the view id before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.view_kind)(self.view.0, &mut out) };
        if status.is_ok() { ScreenKind::from(out) } else { ScreenKind::Other }
    }

    /// Every slot of `collection` as a snapshot, `None` for an empty slot. `None` as a whole for the collection
    /// `Other`, a screen whose container size is not known, a refused read and a loader without
    /// `Area::UI_SCREEN.at(2)` or `Area::INVENTORY_READ.at(3)`.
    pub fn slots(&self, collection: Collection) -> Option<Vec<Option<ItemSnapshot>>> {
        let count = self.slot_count(collection)?;
        let mut all = Vec::with_capacity(usize::try_from(count).ok()?);
        for index in 0..count {
            let slot = SlotAt { collection, index: i32::try_from(index).ok()? };
            let stack = self.item_handle(slot)?;
            all.push(match stack {
                Some(stack) => Some(self.view_of(stack)?.snapshot()),
                None => None,
            });
        }
        Some(all)
    }

    /// The stack on the cursor: `Some(None)` for an empty cursor, `None` for a refused read.
    pub fn cursor(&self) -> Option<Option<ItemSnapshot>> {
        let read = self.table.container_cursor?;
        let mut stack: *const c_void = std::ptr::null();
        // SAFETY: the loader checks the view id before it reads game memory, and `stack` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.view.0, &mut stack) };
        if !status.is_ok() {
            return None;
        }
        if stack.is_null() {
            return Some(None);
        }
        Some(Some(self.view_of(stack)?.snapshot()))
    }

    /// The stack shown in `slot`, as a view that reads like an `Inventory::slot` item. `None` for an empty slot, a
    /// slot outside its collection and a refused read. The view is valid in this event only.
    pub fn visual_item(&self, slot: SlotAt) -> Option<ItemView<'f>> {
        let stack = self.item_handle(slot)??;
        self.view_of(stack)
    }

    /// Moves the whole stack of `slot` to the other side of the screen, as a shift-click does.
    pub fn auto_place(&self, slot: SlotAt) -> Result<(), ApiError> {
        self.act(AmmlContainerAction::AUTO_PLACE, slot, Amount::Whole)
    }

    /// Puts the whole stack of `slot` on the cursor.
    pub fn take_all(&self, slot: SlotAt) -> Result<(), ApiError> {
        self.act(AmmlContainerAction::TAKE_ALL, slot, Amount::Whole)
    }

    /// Puts the cursor stack into `slot`.
    pub fn place_all(&self, slot: SlotAt) -> Result<(), ApiError> {
        self.act(AmmlContainerAction::PLACE_ALL, slot, Amount::Whole)
    }

    /// Drops `amount` of the stack of `slot` out of the screen.
    pub fn drop_slot(&self, slot: SlotAt, amount: Amount) -> Result<(), ApiError> {
        self.act(AmmlContainerAction::DROP_SLOT, slot, amount)
    }

    /// Drops `amount` of the cursor stack out of the screen.
    pub fn drop_cursor(&self, amount: Amount) -> Result<(), ApiError> {
        let slot = SlotAt { collection: Collection::Other, index: 0 };
        self.act(AmmlContainerAction::DROP_CURSOR, slot, amount)
    }

    /// Swaps the stacks of `a` and `b`. `Ok(true)` when the game made the swap, `Ok(false)` when it refused it.
    pub fn swap(&self, a: SlotAt, b: SlotAt) -> Result<bool, ApiError> {
        let swap = self.table.container_swap.ok_or(ApiError::NotAvailable)?;
        let mut swapped = 0u8;
        // SAFETY: the loader checks the view id and the slots before it calls the game, and `swapped` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { swap(self.view.0, a.into(), b.into(), &mut swapped) };
        ApiError::check(status).map(|()| swapped != 0)
    }

    fn act(&self, action: AmmlContainerAction, slot: SlotAt, amount: Amount) -> Result<(), ApiError> {
        let act = self.table.container_action.ok_or(ApiError::NotAvailable)?;
        // SAFETY: the loader checks the view id, the slot and the amount before it calls the game.
        #[allow(unsafe_code)]
        let status = unsafe { act(self.view.0, action, AmmlSlotAt::from(slot), amount.into()) };
        ApiError::check(status)
    }

    fn slot_count(&self, collection: Collection) -> Option<u32> {
        let read = self.table.container_slot_count?;
        let mut count = 0u32;
        // SAFETY: the loader checks the view id before it reads game memory, and `count` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.view.0, collection.raw(), &mut count) };
        status.is_ok().then_some(count)
    }

    // The stack handle of `slot`: `None` for a refused call, `Some(None)` for an empty slot.
    fn item_handle(&self, slot: SlotAt) -> Option<Option<*const c_void>> {
        let read = self.table.container_item?;
        let mut stack: *const c_void = std::ptr::null();
        // SAFETY: the loader checks the view id and the slot before it reads game memory, and `stack` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.view.0, slot.into(), &mut stack) };
        status.is_ok().then_some((!stack.is_null()).then_some(stack))
    }

    // An item view of a stack the loader lent for this event. The null player makes the inventory table accept only a
    // stack lent in the running `ScreenRender` call; `None` for a loader without the inventory table.
    fn view_of(&self, stack: *const c_void) -> Option<ItemView<'f>> {
        Some(ItemView::new(self.inventory?, std::ptr::null(), stack, self.module))
    }
}
