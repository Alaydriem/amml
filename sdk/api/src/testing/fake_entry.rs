use crate::testing::FakeItem;

/// One entry of the contents of a `FakeItem` (a shulker box or a bundle), as `ItemView::container_contents` gives it.
/// `bundle` is the contents of an entry that is a bundle, `bundle_weight` its fill weight.
///
/// The lists are static slices, so a `FakeItem` stays `Copy`: build them in a `const` or with `Box::leak`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeEntry {
    pub slot: u8,
    pub item: FakeItem,
    pub bundle: Option<&'static [FakeEntry]>,
    pub bundle_weight: Option<i32>,
}
