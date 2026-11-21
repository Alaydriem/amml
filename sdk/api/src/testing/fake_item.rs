use amml_abi::AmmlItemSnapshot;

use crate::testing::FakeEntry;

/// An item stack a `FakeGame` serves through `inventory.read` (see `FakeInventory`). `&'static str` keeps it `Copy`.
///
/// `aux`, `max_count`, `is_glint` and `identity` are the version 3 fields: `None` is a field the loader cannot read on
/// the build, so the snapshot gives `None` for it (`identity` is `None` for a stack whose user data cannot be read).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FakeItem {
    pub type_name: &'static str,
    pub count: u8,
    pub aux: Option<i16>,
    pub max_count: Option<u8>,
    pub damage: i32,
    /// 0 for an item with no durability.
    pub max_damage: i32,
    pub is_glint: Option<bool>,
    pub identity: Option<u64>,
    /// The contents of a shulker box or a bundle: what `ItemView::container_contents` gives. `None` is a stack with
    /// no contents list. Needs `Area::INVENTORY_READ.at(4)`.
    pub contents: Option<&'static [FakeEntry]>,
    /// What `ItemView::change_stamp` gives; `None` is a build whose tags the loader cannot read.
    pub change_stamp: Option<u64>,
}

impl FakeItem {
    /// A stack with no name, no count and no version 3 field; a base for a struct update in a constant.
    pub const EMPTY: Self = Self {
        type_name: "",
        count: 0,
        aux: None,
        max_count: None,
        damage: 0,
        max_damage: 0,
        is_glint: None,
        identity: None,
        contents: None,
        change_stamp: None,
    };

    /// The fields of the stack as the loader writes them for `item_read`.
    pub(crate) fn raw_snapshot(&self) -> AmmlItemSnapshot {
        AmmlItemSnapshot {
            count: self.count,
            has_damage: 1,
            damage: self.damage,
            has_max_damage: 1,
            max_damage: self.max_damage,
            has_aux: u8::from(self.aux.is_some()),
            aux: self.aux.unwrap_or(0),
            has_max_count: u8::from(self.max_count.is_some()),
            max_count: self.max_count.unwrap_or(0),
            has_glint: u8::from(self.is_glint.is_some()),
            is_glint: u8::from(self.is_glint == Some(true)),
            has_identity: u8::from(self.identity.is_some()),
            identity: self.identity.unwrap_or(0),
            ..AmmlItemSnapshot::empty()
        }
    }
}
