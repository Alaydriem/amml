use amml_abi::AmmlSlotAt;

use crate::testing::{FakeItem, FakeSlots};
use crate::{Amount, InputMode, Point, Rect, ScreenKind, Size, UiProfile};

/// The screen a `FakeGame` draws for `raise_screen_render`: what the `ScreenView` reads, the `UiDraw` reads and the
/// `ContainerScreen` calls give. A field that is `None` is a read the loader cannot make (it gives `None` in the mod).
/// `can_draw` is whether the game left no images queued: with `false`, `UiDraw::image` is refused.
///
/// A screen whose `kind` is not `Other` is a container screen; `slots` and `cursor` are its stacks. The moves of
/// `ContainerScreen` change them while the dispatch runs and are listed by `FakeGame::screen_actions`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FakeScreen {
    pub kind: ScreenKind,
    pub size: Option<Size>,
    pub content_rect: Option<Rect>,
    pub pointer: Option<Point>,
    pub input_mode: Option<InputMode>,
    pub ui_profile: Option<UiProfile>,
    pub gui_scale: Option<f32>,
    pub can_draw: bool,
    pub slots: FakeSlots,
    pub cursor: Option<FakeItem>,
}

impl FakeScreen {
    /// The stack of `collection` `index`; `None` when the collection has no such slot, `Some(None)` for an empty
    /// slot.
    pub(crate) fn slot(&self, collection: u32, index: i32) -> Option<Option<FakeItem>> {
        let index = usize::try_from(index).ok()?;
        Self::list(&self.slots, collection)?.get(index).copied()
    }

    /// How many slots `collection` has.
    pub(crate) fn slot_count(&self, collection: u32) -> Option<usize> {
        Self::list(&self.slots, collection).map(Vec::len)
    }

    fn list(slots: &FakeSlots, collection: u32) -> Option<&Vec<Option<FakeItem>>> {
        match collection {
            AmmlSlotAt::CONTAINER => Some(&slots.container),
            AmmlSlotAt::HOTBAR => Some(&slots.hotbar),
            AmmlSlotAt::INVENTORY => Some(&slots.inventory),
            _ => None,
        }
    }

    fn cell(&mut self, slot: AmmlSlotAt) -> Option<&mut Option<FakeItem>> {
        let index = usize::try_from(slot.index).ok()?;
        match slot.collection {
            AmmlSlotAt::CONTAINER => self.slots.container.get_mut(index),
            AmmlSlotAt::HOTBAR => self.slots.hotbar.get_mut(index),
            AmmlSlotAt::INVENTORY => self.slots.inventory.get_mut(index),
            _ => None,
        }
    }

    /// A shift-click: the stack of `slot` goes to the first empty slot of the other side (the hotbar, then the
    /// inventory, for a container slot; the container for a player slot). Nothing moves when no slot is free.
    pub(crate) fn auto_place(&mut self, slot: AmmlSlotAt) {
        let Some(item) = self.cell(slot).and_then(|cell| *cell) else { return };
        let targets: &[u32] = if slot.collection == AmmlSlotAt::CONTAINER {
            &[AmmlSlotAt::HOTBAR, AmmlSlotAt::INVENTORY]
        } else {
            &[AmmlSlotAt::CONTAINER]
        };
        for collection in targets {
            let Some(count) = self.slot_count(*collection) else { continue };
            for index in 0..count {
                let target = AmmlSlotAt { collection: *collection, index: i32::try_from(index).unwrap_or(i32::MAX) };
                let placed = match self.cell(target) {
                    Some(cell) if cell.is_none() => {
                        *cell = Some(item);
                        true
                    }
                    _ => false,
                };
                if placed {
                    if let Some(cell) = self.cell(slot) {
                        *cell = None;
                    }
                    return;
                }
            }
        }
    }

    /// The stack of `slot` goes onto an empty cursor.
    pub(crate) fn take_all(&mut self, slot: AmmlSlotAt) {
        if self.cursor.is_some() {
            return;
        }
        let taken = self.cell(slot).and_then(Option::take);
        self.cursor = taken;
    }

    /// The cursor stack goes into `slot`; a stack already there goes onto the cursor.
    pub(crate) fn place_all(&mut self, slot: AmmlSlotAt) {
        let held = self.cursor;
        let Some(cell) = self.cell(slot) else { return };
        let before = std::mem::replace(cell, held);
        self.cursor = before;
    }

    /// `amount` of the stack of `slot` is dropped.
    pub(crate) fn drop_slot(&mut self, slot: AmmlSlotAt, amount: Amount) {
        if let Some(cell) = self.cell(slot) {
            Self::remove(cell, amount);
        }
    }

    /// `amount` of the cursor stack is dropped.
    pub(crate) fn drop_cursor(&mut self, amount: Amount) {
        Self::remove(&mut self.cursor, amount);
    }

    /// The stacks of `a` and `b` change places when both hold a stack; whether they did.
    pub(crate) fn swap(&mut self, a: AmmlSlotAt, b: AmmlSlotAt) -> bool {
        let (Some(first), Some(second)) = (self.cell(a).and_then(|cell| *cell), self.cell(b).and_then(|cell| *cell))
        else {
            return false;
        };
        if let Some(cell) = self.cell(a) {
            *cell = Some(second);
        }
        if let Some(cell) = self.cell(b) {
            *cell = Some(first);
        }
        true
    }

    fn remove(cell: &mut Option<FakeItem>, amount: Amount) {
        let Some(item) = cell.as_mut() else { return };
        let removed = match amount {
            Amount::Whole => item.count,
            Amount::One => 1,
            Amount::Count(count) => count,
        };
        item.count = item.count.saturating_sub(removed);
        if item.count == 0 {
            *cell = None;
        }
    }
}
