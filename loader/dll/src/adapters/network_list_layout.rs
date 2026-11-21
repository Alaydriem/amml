use amml_abi::PlayerListUpdate;

use crate::{LoaderState, ObjectRead};

/// Where the entries of a `PlayerListPacket` are and what one entry holds, from the entries
/// `PlayerListPacket.mEntries`, `PlayerListAddEntry.size`, `PlayerListPacket.variantIndex` and the members of an Add entry.
///
/// The entries are `std::variant<RemoveEntry, AddEntry>` elements of one vector. Both alternatives keep the uuid at the
/// same place; only an Add entry has the actor id, the skin, the name and the Xbox user id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkListLayout {
    pub entries: usize,
    pub size: usize,
    pub variant: usize,
    pub uuid: usize,
    pub actor: usize,
    pub skin: usize,
    pub name: usize,
    pub xuid: usize,
}

impl NetworkListLayout {
    /// The variant index of a Remove entry.
    pub const REMOVE: u8 = 0;
    /// The variant index of an Add entry.
    pub const ADD: u8 = 1;
    /// The size of a string member of an entry (`std::string`).
    const STRING_SIZE: usize = 0x20;
    /// The size of a `SerializedSkinRef` (a `shared_ptr`).
    const SKIN_REF_SIZE: usize = 0x10;

    /// The layout of the resolved entries; `None` when one is missing or the members do not fit inside one element.
    pub fn from_state(state: &LoaderState) -> Option<Self> {
        let field = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let layout = Self {
            entries: field("PlayerListPacket.mEntries")?,
            size: field("PlayerListAddEntry.size")?,
            variant: field("PlayerListPacket.variantIndex")?,
            uuid: field("PlayerListAddEntry.mUUID")?,
            actor: field("PlayerListAddEntry.mId")?,
            skin: field("PlayerListAddEntry.mSkin")?,
            name: field("PlayerListAddEntry.mName")?,
            xuid: field("PlayerListAddEntry.mXUID")?,
        };
        layout.fits().then_some(layout)
    }

    // Whether every member read lies inside one element.
    fn fits(&self) -> bool {
        let inside = |at: usize, length: usize| at.checked_add(length).is_some_and(|end| end <= self.size);
        self.size != 0
            && inside(self.variant, 1)
            && inside(self.uuid, 16)
            && inside(self.actor, 8)
            && inside(self.skin, Self::SKIN_REF_SIZE)
            && inside(self.name, Self::STRING_SIZE)
            && inside(self.xuid, Self::STRING_SIZE)
    }

    /// The elements of the packet that carry the action of its first element, as addresses, at most
    /// `PlayerListUpdate::MAX_ENTRIES`; the action is `PlayerListUpdate::ADD` or `REMOVE`. `None` for a vector that does
    /// not look sane (its byte size is not a multiple of the element size, or it has no first element). The second
    /// value is the number of elements the cut left out.
    ///
    /// # Safety
    /// `packet` is the live `PlayerListPacket` the game gave its handler.
    pub unsafe fn elements(&self, packet: usize) -> Option<(u32, Vec<usize>, usize)> {
        // SAFETY: the caller guarantees a live packet; the vector (begin, end) sits at `entries`.
        let [begin, end] = unsafe { ObjectRead::value::<[usize; 2]>(packet, self.entries) }?;
        let bytes = end.checked_sub(begin)?;
        if begin == 0 || bytes == 0 || bytes % self.size != 0 {
            return None;
        }
        let total = bytes / self.size;
        let kept = total.min(PlayerListUpdate::MAX_ENTRIES as usize);
        // SAFETY: element 0 lies inside the live vector, and the variant index is inside it.
        let first = unsafe { ObjectRead::value::<u8>(begin, self.variant) }?;
        let (action, variant) = match first {
            Self::ADD => (PlayerListUpdate::ADD, Self::ADD),
            Self::REMOVE => (PlayerListUpdate::REMOVE, Self::REMOVE),
            _ => return None,
        };
        let mut elements = Vec::with_capacity(kept);
        for index in 0..kept {
            let element = begin.checked_add(index.checked_mul(self.size)?)?;
            // SAFETY: the element lies inside the live vector, and the variant index is inside it.
            if unsafe { ObjectRead::value::<u8>(element, self.variant) } == Some(variant) {
                elements.push(element);
            }
        }
        Some((action, elements, total - kept))
    }
}
