use crate::{HashedStringLayout30, HashedStringLayout38};

/// Reads the text of a game `HashedString` (`Item::mFullName`, `BlockType::mNameInfo.mFullName`) through the
/// mirror that `HashedString.layout` names: 0 the 30h layout, 1 the 38h layout. Any other value reads nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameReader {
    layout: u64,
}

impl NameReader {
    pub const LAYOUT_30: u64 = 0;
    pub const LAYOUT_38: u64 = 1;

    /// A reader for the derived `HashedString.layout` value; `None` for a value no mirror has.
    pub fn new(layout: u64) -> Option<Self> {
        matches!(layout, Self::LAYOUT_30 | Self::LAYOUT_38).then_some(Self { layout })
    }

    /// The text of the `HashedString` at `address`; `None` for a null address, a variant index the mirror does not
    /// know, or a text that is not UTF-8.
    ///
    /// # Safety
    /// A non-zero `address` is a live game `HashedString` of the layout this reader was made for, read on the
    /// thread that owns it.
    pub unsafe fn read(&self, address: usize) -> Option<String> {
        if address == 0 {
            return None;
        }
        let bytes = if self.layout == Self::LAYOUT_30 {
            // SAFETY: the caller guarantees a live HashedString of the 30h layout.
            let name = unsafe { &*std::ptr::with_exposed_provenance::<HashedStringLayout30>(address) };
            name.text.as_bytes()
        } else {
            // SAFETY: the caller guarantees a live HashedString of the 38h layout.
            let name = unsafe { &*std::ptr::with_exposed_provenance::<HashedStringLayout38>(address) };
            // SAFETY: the game keeps the text of a live HashedString readable.
            unsafe { name.text() }?
        };
        std::str::from_utf8(bytes).ok().map(str::to_owned)
    }
}
