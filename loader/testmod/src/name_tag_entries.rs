use std::ffi::c_void;

use amml_game::{EntryId, EntryNeed, GameEntries};

/// The entries of the name-tag list that the probe reads.
pub struct NameTagEntries;

impl NameTagEntries {
    /// The primary vtable of `LevelRendererPlayer`.
    pub const VTABLE: EntryId = EntryId::new("LevelRendererPlayer.vtable");
    /// The slot of `extractNameTags` in that vtable.
    pub const EXTRACT: EntryId = EntryId::new("LevelRendererPlayer.extractNameTags");
    /// The size of one `NameTagRenderObject`.
    pub const SIZE: EntryId = EntryId::new("NameTagRenderObject.size");

    /// The three entries, all under the feature `labels`.
    pub fn wanted() -> Vec<(EntryId, EntryNeed)> {
        let labels = EntryNeed::Optional { feature: "labels" };
        vec![(Self::VTABLE, labels), (Self::EXTRACT, labels), (Self::SIZE, labels)]
    }

    /// The address of the `extractNameTags` function: the code pointer in its slot of the vtable. `None` when an
    /// entry is off or the slot is null.
    #[allow(unsafe_code)]
    pub fn target(entries: &GameEntries) -> Option<u64> {
        let vtable = usize::try_from(entries.get(Self::VTABLE)?).ok()?;
        let slot = usize::try_from(entries.get(Self::EXTRACT)?).ok()?;
        if vtable == 0 {
            return None;
        }
        let at = (vtable as *const *const c_void).wrapping_add(slot);
        // SAFETY: the vtable entry is the address of the primary vtable of the class in the game image, which
        // holds at least `slot + 1` code pointers; the slot is read once at enable time.
        let function = unsafe { at.read() };
        (!function.is_null()).then_some(function as u64)
    }
}
