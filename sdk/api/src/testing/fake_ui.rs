use std::cell::RefCell;

use crate::testing::FakeItem;
use crate::testing::fake_screen::FakeScreen;

/// The ui side of a `FakeGame`: the screens whose `ScreenRender` dispatch runs now, innermost last, each with its
/// view id and screen id. A ui table call answers only for the view of the innermost one, as the loader does.
#[derive(Default)]
pub(crate) struct FakeUi {
    running: RefCell<Vec<(u64, u64, FakeScreen)>>,
    draws: RefCell<Vec<String>>,
    actions: RefCell<Vec<String>>,
    // The stack handles lent in the running dispatch; a move forgets them, as the loader does.
    lent: RefCell<Vec<usize>>,
    // The `Inventory::slot` handles that were lent while a dispatch ran. Such a handle is valid only while it is in
    // `lent`: a move, the end of the dispatch or a later dispatch refuses it.
    scoped: RefCell<Vec<usize>>,
    // The hold-backs of cancelled pointer downs, as the loader keeps them.
    pointers: amml_runtime::PointerUpGate,
}

impl FakeUi {
    // The stack handles of a container screen are this far from zero, so no handle of another kind equals one.
    const HANDLE_BASE: usize = 0x7000_0000;
    const COLLECTION_STRIDE: usize = 0x1000;
    const SLOT_STRIDE: usize = 8;
    const CURSOR_HANDLE: usize = Self::HANDLE_BASE + 0x8000;

    /// A `ScreenRender` dispatch of `view` of the screen with id `screen` starts.
    pub(crate) fn enter(&self, view: u64, screen_id: u64, screen: FakeScreen) {
        self.running.borrow_mut().push((view, screen_id, screen));
        self.lent.borrow_mut().clear();
    }

    /// Records one draw call.
    pub(crate) fn record_draw(&self, line: String) {
        self.draws.borrow_mut().push(line);
    }

    /// The draw calls so far.
    pub(crate) fn draws(&self) -> Vec<String> {
        self.draws.borrow().clone()
    }

    /// Records one container move. The stacks lent before it are refused after it.
    pub(crate) fn record_action(&self, line: String) {
        self.actions.borrow_mut().push(line);
        self.forget();
    }

    /// The loader forgets the stacks it lent in the running dispatch (a move of a container screen, or a request that
    /// predicts a swap).
    pub(crate) fn forget(&self) {
        self.lent.borrow_mut().clear();
    }

    /// Notes that `Inventory::slot` gave `handle`: while a dispatch runs the handle is lent in it, outside a dispatch
    /// it is valid for the frame.
    pub(crate) fn lend_slot(&self, handle: usize) {
        if handle == 0 {
            return;
        }
        if self.running.borrow().is_empty() {
            self.scoped.borrow_mut().retain(|scoped| *scoped != handle);
            return;
        }
        self.lent.borrow_mut().push(handle);
        self.scoped.borrow_mut().push(handle);
    }

    /// Whether the loader still accepts the `Inventory::slot` handle `handle`.
    pub(crate) fn slot_lend_valid(&self, handle: usize) -> bool {
        !self.scoped.borrow().contains(&handle)
            || (!self.running.borrow().is_empty() && self.lent.borrow().contains(&handle))
    }

    /// The container moves so far.
    pub(crate) fn actions(&self) -> Vec<String> {
        self.actions.borrow().clone()
    }

    /// The innermost dispatch ends; gives its screen with the moves the dispatch made.
    pub(crate) fn leave(&self) -> Option<FakeScreen> {
        self.running.borrow_mut().pop().map(|(_, _, screen)| screen)
    }

    /// The screen of `view` when its dispatch is the innermost one.
    pub(crate) fn screen_of(&self, view: u64) -> Option<FakeScreen> {
        self.running.borrow().last().filter(|(running, _, _)| *running == view).map(|(_, _, screen)| screen.clone())
    }

    /// The screen id of `view` when its dispatch is the innermost one.
    pub(crate) fn screen_id_of(&self, view: u64) -> Option<u64> {
        self.running.borrow().last().filter(|(running, _, _)| *running == view).map(|(_, id, _)| *id)
    }

    /// Runs `change` on the screen of `view` when its dispatch is the innermost one.
    pub(crate) fn change<R>(&self, view: u64, change: impl FnOnce(&mut FakeScreen) -> R) -> Option<R> {
        let mut running = self.running.borrow_mut();
        let (_, _, screen) = running.last_mut().filter(|(running, _, _)| *running == view)?;
        Some(change(screen))
    }

    /// Notes that the loader lent `handle` in the running dispatch.
    pub(crate) fn lend(&self, handle: usize) {
        if handle != 0 {
            self.lent.borrow_mut().push(handle);
        }
    }

    /// The handle of the stack in `collection` slot `index`; never read as memory.
    pub(crate) fn slot_handle(collection: u32, index: i32) -> usize {
        let index = usize::try_from(index).unwrap_or(0);
        Self::HANDLE_BASE
            + (collection as usize) * Self::COLLECTION_STRIDE
            + index * Self::SLOT_STRIDE
            + Self::SLOT_STRIDE
    }

    /// The handle of the cursor stack; never read as memory.
    pub(crate) fn cursor_handle() -> usize {
        Self::CURSOR_HANDLE
    }

    /// The stack a handle names in the innermost running screen; `None` for a handle that names no stack now.
    pub(crate) fn item_at_handle(&self, handle: usize) -> Option<FakeItem> {
        let running = self.running.borrow();
        let (_, _, screen) = running.last()?;
        if !self.lent.borrow().contains(&handle) {
            return None;
        }
        if handle == Self::CURSOR_HANDLE {
            return screen.cursor;
        }
        let offset = handle.checked_sub(Self::HANDLE_BASE)?;
        let collection = u32::try_from(offset / Self::COLLECTION_STRIDE).ok()?;
        let within = offset % Self::COLLECTION_STRIDE;
        if within % Self::SLOT_STRIDE != 0 || within == 0 {
            return None;
        }
        let index = i32::try_from(within / Self::SLOT_STRIDE - 1).ok()?;
        screen.slot(collection, index)?
    }
}

impl FakeUi {
    /// The gate of the pointer buttons.
    pub(crate) fn pointers(&self) -> &amml_runtime::PointerUpGate {
        &self.pointers
    }
}
