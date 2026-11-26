use crate::{AmmlItemDraw, AmmlItemSnapshot};

/// One entry of the contents of a shulker box or a bundle, as `AmmlInventoryTable::contents_entry` writes it.
///
/// `item` holds the fields of the stack the entry makes (its `size` and `version` are those of a v1 snapshot).
/// `draw` is a copy of that stack for `AmmlUiTable::draw_item`; the caller owns it and frees it with
/// `item_draw_release`. `bundle` is the contents id of the bundle the entry is, 0 for an entry that holds none; the
/// caller closes it with `contents_close`. `has_bundle_weight` says that `bundle_weight` was read.
///
/// The struct only grows: a field added later is appended, `size` and `version` grow, and the loader writes only as
/// many bytes as the caller's `size` covers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlContainerEntry {
    pub size: u32,
    pub version: u32,
    pub slot: u8,
    pub has_bundle_weight: u8,
    pub _reserved: [u8; 2],
    pub bundle_weight: i32,
    pub item: AmmlItemSnapshot,
    pub draw: AmmlItemDraw,
    pub bundle: u64,
}

impl AmmlContainerEntry {
    /// `size` of a v1 entry. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 80;

    /// An empty v1 entry for the loader to fill.
    pub const fn empty() -> Self {
        Self {
            size: Self::SIZE_V1,
            version: 1,
            slot: 0,
            has_bundle_weight: 0,
            _reserved: [0; 2],
            bundle_weight: 0,
            item: AmmlItemSnapshot::empty(),
            draw: AmmlItemDraw { id: 0, generation: 0, _reserved: 0 },
            bundle: 0,
        }
    }
}

const _: () = assert!(core::mem::size_of::<AmmlContainerEntry>() == AmmlContainerEntry::SIZE_V1 as usize);
