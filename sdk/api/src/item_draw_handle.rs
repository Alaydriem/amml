use amml_abi::{AmmlInventoryTable, AmmlItemDraw, AmmlStr};

/// A copy of one game item stack that the loader keeps for drawing: `UiDraw::item` draws it. It is owned, not
/// `Clone`, and `Send`: it holds no game pointer. Dropping it frees the copy; the loader destroys the game stack at
/// its next UI draw, so a handle may be dropped on any thread.
///
/// A mod holds at most 4096 handles at a time, and one `ItemView::container_contents` call makes at most 1755 (the 27
/// slots of a box and 64 items for each of them as a bundle); past either the call leaves out the contents of the last
/// bundles, and gives `None` when the top level alone does not fit. A handle of a
/// world that was left is `Stale` for `UiDraw::item`; dropping it is still allowed.
pub struct ItemDrawHandle {
    table: AmmlInventoryTable,
    module: &'static str,
    draw: AmmlItemDraw,
}

impl ItemDrawHandle {
    pub(crate) fn new(table: AmmlInventoryTable, module: &'static str, draw: AmmlItemDraw) -> Self {
        Self { table, module, draw }
    }

    /// The draw id and the module it belongs to, for the ui table call.
    pub(crate) fn raw(&self) -> (&'static str, AmmlItemDraw) {
        (self.module, self.draw)
    }
}

impl std::fmt::Debug for ItemDrawHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ItemDrawHandle")
            .field("module", &self.module)
            .field("id", &self.draw.id)
            .field("generation", &self.draw.generation)
            .finish()
    }
}

impl Drop for ItemDrawHandle {
    fn drop(&mut self) {
        let Some(release) = self.table.item_draw_release else { return };
        // SAFETY: the loader checks the draw at once and only queues the free of the game stack; the module id lives
        // for the call.
        #[allow(unsafe_code)]
        let _ = unsafe { release(AmmlStr::from_str(self.module), self.draw) };
    }
}
