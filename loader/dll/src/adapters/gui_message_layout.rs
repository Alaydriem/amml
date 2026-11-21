use amml_abi::{AmmlChatField, AmmlStatus};
use amml_msvc::{MsvcHeap, MsvcString};

use crate::{LoaderState, ObjectRead};

/// Where the members of a game `GuiMessage` are, from the entries `GuiMessage.size`, `GuiMessage.mType`,
/// `GuiMessage.mUsername`, `GuiMessage.mFullString`, `GuiMessage.mFilteredFullString`,
/// `GuiMessage.mFilteredFullStringEngaged` and `GuiMessage.mAuthorXuid`.
///
/// The object is 0x110 bytes on 26.63 and 0x118 bytes on 70.26; the six members sit at the same offsets on both, so one
/// set of offsets serves both sizes. The layout is refused for any other size, because a size no mirror has means the
/// members may have moved. Every string is an MSVC `std::string` of 32 bytes; the filtered full string is the value of
/// a `std::optional<std::string>` whose engaged byte is `filtered_engaged`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuiMessageLayout {
    pub size: usize,
    pub kind: usize,
    pub username: usize,
    pub full_text: usize,
    pub filtered_text: usize,
    pub filtered_engaged: usize,
    pub xuid: usize,
}

impl GuiMessageLayout {
    /// The sizes of `GuiMessage` the loader has a mirror for.
    pub const SIZES: [usize; 2] = [0x110, 0x118];
    /// The bytes of one `std::string`.
    pub const STRING_BYTES: usize = 32;
    /// The longest text the loader reads from a line; a string with a larger capacity is not believed.
    pub const MAX_TEXT: usize = 1 << 20;
    // The size and the capacity words of a `std::string`, after its 16 bytes of storage.
    const LENGTH_AT: usize = 16;
    const CAPACITY_AT: usize = 24;
    // The longest text that stays in the storage of the string.
    const SMALL_CAPACITY: usize = 15;

    /// The layout of the resolved entries; `None` when one is missing or the members do not fit a mirror.
    pub fn from_state(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        Self::new(
            value("GuiMessage.size")?,
            value("GuiMessage.mType")?,
            value("GuiMessage.mUsername")?,
            value("GuiMessage.mFullString")?,
            value("GuiMessage.mFilteredFullString")?,
            value("GuiMessage.mFilteredFullStringEngaged")?,
            value("GuiMessage.mAuthorXuid")?,
        )
    }

    /// A layout for an object of `size` bytes; `None` for a size without a mirror, a member that is not inside the
    /// object, a string that is not 8-aligned, or members that overlap.
    pub fn new(
        size: usize,
        kind: usize,
        username: usize,
        full_text: usize,
        filtered_text: usize,
        filtered_engaged: usize,
        xuid: usize,
    ) -> Option<Self> {
        let layout = Self { size, kind, username, full_text, filtered_text, filtered_engaged, xuid };
        layout.fits().then_some(layout)
    }

    // The byte ranges the members take: the type, the three strings, the filtered string and the engaged byte.
    fn ranges(&self) -> [(usize, usize); 6] {
        let string = |at: usize| (at, at.saturating_add(Self::STRING_BYTES));
        [
            (self.kind, self.kind.saturating_add(size_of::<u32>())),
            string(self.username),
            string(self.full_text),
            string(self.filtered_text),
            (self.filtered_engaged, self.filtered_engaged.saturating_add(1)),
            string(self.xuid),
        ]
    }

    fn fits(&self) -> bool {
        if !Self::SIZES.contains(&self.size) {
            return false;
        }
        let aligned = [self.username, self.full_text, self.filtered_text, self.xuid]
            .iter()
            .all(|at| at.is_multiple_of(size_of::<usize>()));
        let ranges = self.ranges();
        let inside = ranges.iter().all(|(start, end)| *end <= self.size && start < end);
        let apart = ranges.iter().enumerate().all(|(index, (start, end))| {
            ranges.iter().skip(index + 1).all(|(other_start, other_end)| end <= other_start || other_end <= start)
        });
        aligned && inside && apart
    }

    /// The message type of the line at `message`.
    ///
    /// # Safety
    /// `message` is the address of a live game `GuiMessage` of the size this layout was made for.
    pub unsafe fn kind_of(&self, message: usize) -> Option<u32> {
        // SAFETY: the caller's guarantee is passed on; the type is the first member.
        unsafe { ObjectRead::value::<u32>(message, self.kind) }
    }

    /// Whether the line at `message` has a filtered full string (the engaged byte of the optional is 1).
    ///
    /// # Safety
    /// `message` is the address of a live game `GuiMessage` of the size this layout was made for.
    pub unsafe fn has_filtered(&self, message: usize) -> bool {
        // SAFETY: the caller's guarantee is passed on; the engaged byte is inside the object.
        let engaged = unsafe { ObjectRead::value::<u8>(message, self.filtered_engaged) };
        engaged == Some(1)
    }

    // The offset of the string of `field` in the object.
    fn offset_of(&self, field: AmmlChatField) -> Option<usize> {
        match field {
            AmmlChatField::USERNAME => Some(self.username),
            AmmlChatField::XUID => Some(self.xuid),
            AmmlChatField::TEXT => Some(self.full_text),
            AmmlChatField::FILTERED_TEXT => Some(self.filtered_text),
            _ => None,
        }
    }

    /// The address and the length of the bytes of one string of the line at `message`. The bytes are the game's own,
    /// valid while the message lives, and valid UTF-8.
    ///
    /// `INVALID_ARGUMENT` for a field the loader does not know and for a string whose size and capacity are not
    /// believable or whose bytes are not UTF-8; `NOT_AVAILABLE` for the filtered text of a line that has none.
    ///
    /// # Safety
    /// `message` is the address of a live game `GuiMessage` of the size this layout was made for.
    pub unsafe fn text_of(&self, message: usize, field: AmmlChatField) -> Result<(usize, usize), AmmlStatus> {
        let offset = self.offset_of(field).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        if field == AmmlChatField::FILTERED_TEXT {
            // SAFETY: the caller's guarantee is passed on; the engaged byte is inside the object.
            let engaged = unsafe { ObjectRead::value::<u8>(message, self.filtered_engaged) };
            if engaged != Some(1) {
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
        }
        let string = message.checked_add(offset).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        // SAFETY: the string is inside the live object; its size and capacity words follow its 16 bytes of storage.
        let length = unsafe { ObjectRead::value::<usize>(string, Self::LENGTH_AT) };
        // SAFETY: as above.
        let capacity = unsafe { ObjectRead::value::<usize>(string, Self::CAPACITY_AT) };
        let (Some(length), Some(capacity)) = (length, capacity) else { return Err(AmmlStatus::INVALID_ARGUMENT) };
        if length > capacity || capacity > Self::MAX_TEXT {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let data = if capacity <= Self::SMALL_CAPACITY {
            string
        } else {
            // SAFETY: a string with a capacity above the storage holds its data pointer in the first word.
            unsafe { ObjectRead::pointer(string, 0) }.ok_or(AmmlStatus::INVALID_ARGUMENT)?
        };
        if length == 0 {
            return Ok((data, 0));
        }
        // SAFETY: the string holds `length` initialised bytes at `data`, which the game keeps while the message lives.
        let bytes = unsafe { std::slice::from_raw_parts(std::ptr::with_exposed_provenance::<u8>(data), length) };
        std::str::from_utf8(bytes).map_err(|_| AmmlStatus::INVALID_ARGUMENT)?;
        Ok((data, length))
    }

    /// Replaces one string of the line at `message` with `text`, built on `heap`; the old block is freed after the new
    /// one exists, so a failed allocation changes nothing. False for `USERNAME`, `XUID` and unknown fields (read
    /// only), for the filtered text of a line that has none, and when the heap gives no block.
    ///
    /// # Safety
    /// `message` is the address of a live game `GuiMessage` of the size this layout was made for, whose strings were
    /// built by the allocator `heap` calls, and no other thread uses it during the call.
    pub unsafe fn replace(&self, message: usize, field: AmmlChatField, heap: &MsvcHeap, text: &str) -> bool {
        if field != AmmlChatField::TEXT && field != AmmlChatField::FILTERED_TEXT {
            return false;
        }
        if field == AmmlChatField::FILTERED_TEXT {
            // SAFETY: the caller's guarantee is passed on; the engaged byte is inside the object.
            if unsafe { ObjectRead::value::<u8>(message, self.filtered_engaged) } != Some(1) {
                return false;
            }
        }
        let Some(offset) = self.offset_of(field) else { return false };
        // SAFETY: `heap` is the game `operator new` and `operator delete`.
        let Ok(new) = (unsafe { MsvcString::new_in(heap, text) }) else { return false };
        let string = std::ptr::with_exposed_provenance_mut::<MsvcString>(message + offset);
        // SAFETY: the string is inside the live object, 8-aligned (`fits`), and the caller guarantees its block came
        // from `heap`; it is not used again after the release.
        unsafe {
            (*string).release(heap);
            string.write(new);
        }
        true
    }
}
