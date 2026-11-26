/// A game item stack copy that the loader keeps for drawing (`AmmlUiTable::draw_item`). `id` is never given twice in a
/// process. `generation` counts the world leaves: a draw of an older generation is `STALE`. The mod frees the draw
/// with `AmmlInventoryTable::item_draw_release`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlItemDraw {
    pub id: u64,
    pub generation: u32,
    pub _reserved: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlItemDraw>() == 16);
