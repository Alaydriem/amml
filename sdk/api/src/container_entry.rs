use crate::{ContainerContents, ItemDrawHandle, ItemSnapshot};

/// One entry of the contents of a shulker box or a bundle, copied by `ItemView::container_contents`.
///
/// `slot` is the `Slot` byte of the entry in the tag (the position in the list when the tag has none). `item` holds
/// the fields of the stack; `draw` draws it with `UiDraw::item`. A bundle in the contents has its own contents in
/// `bundle` (`None` for a bundle with no contents list and for an item that is no bundle) and its fill weight in
/// `bundle_weight`.
#[derive(Debug)]
pub struct ContainerEntry {
    pub slot: u8,
    pub item: ItemSnapshot,
    pub draw: ItemDrawHandle,
    pub bundle: Option<ContainerContents>,
    pub bundle_weight: Option<i32>,
}
