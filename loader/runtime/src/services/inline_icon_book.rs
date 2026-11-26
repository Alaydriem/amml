use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::structs::inline_icon_entry::InlineIconEntry;
use crate::structs::inline_icon_state::InlineIconState;
use crate::{GlyphPage, IconKey, IconRefusal};

/// The inline icons of the chat font: 240 cells, the code points `U+E110..=U+E1FF`, each with an image of at most
/// 16 x 16 RGBA (drawn from the top left of its cell) and an advance in GUI units.
///
/// The book owns the glyph page. A change of an icon changes the page and marks it dirty; the upload side takes it
/// with `take_page`. The book makes no game call, so every call is allowed from any thread.
///
/// A cell is handed out in order, and a released cell is blanked. A released cell is used again only when every cell
/// was used once, and then the one released first goes first: a code point that is still in a chat line shows blank
/// for as long as possible instead of showing another icon. `world_left` frees every icon and makes the old keys
/// stale.
///
/// `advance_of` and `has_icons` read atomics, so the width hook of the font takes no lock.
pub struct InlineIconBook {
    state: Mutex<InlineIconState>,
    // The advance of each cell as `f32` bits; 0 for a cell with no icon (an advance is always above 0).
    advances: Vec<AtomicU32>,
    count: AtomicUsize,
}

impl InlineIconBook {
    /// The number of icon cells.
    pub const CELLS: usize = 240;
    /// The first icon code point.
    pub const FIRST_CODE_POINT: u32 = GlyphPage::FIRST_CELL;
    /// The longest side of an icon image in pixels.
    pub const MAX_SIDE: u32 = GlyphPage::CELL_SIDE;
    /// The largest advance, in GUI units.
    pub const MAX_ADVANCE: f32 = 16.0;

    /// A book with no icon over a glyph page that holds `row0` as its vanilla row.
    pub fn new(row0: &[u8]) -> Self {
        Self {
            state: Mutex::new(InlineIconState {
                generation: 0,
                next_id: 1,
                icons: BTreeMap::new(),
                used: vec![false; Self::CELLS],
                released: VecDeque::new(),
                page: GlyphPage::new(row0),
            }),
            advances: (0..Self::CELLS).map(|_| AtomicU32::new(0)).collect(),
            count: AtomicUsize::new(0),
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, InlineIconState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn code_point(cell: usize) -> u32 {
        Self::FIRST_CODE_POINT + cell as u32
    }

    fn check_image(width: u32, height: u32, pixels: &[u8]) -> Result<(), IconRefusal> {
        if width == 0 || height == 0 || u64::try_from(pixels.len()) != Ok(u64::from(width) * u64::from(height) * 4) {
            return Err(IconRefusal::BadImage);
        }
        if width > Self::MAX_SIDE || height > Self::MAX_SIDE {
            return Err(IconRefusal::Refused);
        }
        Ok(())
    }

    // The entry of `key` when it is current and belongs to `owner`.
    fn entry(state: &InlineIconState, owner: usize, key: IconKey) -> Result<InlineIconEntry, IconRefusal> {
        if key.generation != state.generation {
            return Err(IconRefusal::Stale);
        }
        let entry = state.icons.get(&key.id).ok_or(IconRefusal::Stale)?;
        if entry.owner != owner {
            return Err(IconRefusal::Refused);
        }
        Ok(*entry)
    }

    /// Registers the image as a new icon of `owner`, drawn `advance` GUI units wide. `Refused` for an advance that is
    /// not above 0 and at most 16 (a value that is not finite included), an image over 16 x 16, and a book with all
    /// 240 cells in use; `BadImage` for a size of 0 or pixels that do not fit it.
    pub fn register(
        &self,
        owner: usize,
        width: u32,
        height: u32,
        pixels: &[u8],
        advance: f32,
    ) -> Result<IconKey, IconRefusal> {
        Self::check_image(width, height, pixels)?;
        if !(advance > 0.0 && advance <= Self::MAX_ADVANCE) {
            return Err(IconRefusal::Refused);
        }
        let mut state = self.lock();
        let cell = match state.used.iter().position(|used| !used) {
            Some(cell) => cell,
            None => state.released.pop_front().ok_or(IconRefusal::Refused)?,
        };
        state.used[cell] = true;
        let code_point = Self::code_point(cell);
        if !state.page.write_cell(code_point, width, height, pixels) {
            return Err(IconRefusal::Refused);
        }
        let id = state.next_id;
        state.next_id += 1;
        state.icons.insert(id, InlineIconEntry { owner, cell });
        self.advances[cell].store(advance.to_bits(), Ordering::Release);
        self.count.fetch_add(1, Ordering::AcqRel);
        Ok(IconKey { id, generation: state.generation, code_point })
    }

    /// Replaces the image of `key` of `owner`; the advance stays.
    pub fn update(
        &self,
        owner: usize,
        key: IconKey,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<(), IconRefusal> {
        let mut state = self.lock();
        let entry = Self::entry(&state, owner, key)?;
        Self::check_image(width, height, pixels)?;
        if state.page.write_cell(Self::code_point(entry.cell), width, height, pixels) {
            Ok(())
        } else {
            Err(IconRefusal::Refused)
        }
    }

    /// Frees `key` of `owner`: its cell is blanked and can be used again when no unused cell is left.
    pub fn release(&self, owner: usize, key: IconKey) -> Result<(), IconRefusal> {
        let mut state = self.lock();
        let entry = Self::entry(&state, owner, key)?;
        self.free(&mut state, key.id, entry);
        Ok(())
    }

    // Takes one icon out of the book and blanks its cell.
    fn free(&self, state: &mut InlineIconState, id: u64, entry: InlineIconEntry) {
        state.icons.remove(&id);
        state.page.clear_cell(Self::code_point(entry.cell));
        state.released.push_back(entry.cell);
        self.advances[entry.cell].store(0, Ordering::Release);
        self.count.fetch_sub(1, Ordering::AcqRel);
    }

    /// Frees every icon of `owner`. Gives the number of icons freed.
    pub fn disable_owner(&self, owner: usize) -> usize {
        let mut state = self.lock();
        let ids: Vec<(u64, InlineIconEntry)> =
            state.icons.iter().filter(|(_, entry)| entry.owner == owner).map(|(id, entry)| (*id, *entry)).collect();
        for (id, entry) in &ids {
            self.free(&mut state, *id, *entry);
        }
        ids.len()
    }

    /// A world was left: every icon is freed and the old keys are stale. Gives the number of icons freed.
    pub fn world_left(&self) -> usize {
        let mut state = self.lock();
        let ids: Vec<(u64, InlineIconEntry)> = state.icons.iter().map(|(id, entry)| (*id, *entry)).collect();
        for (id, entry) in &ids {
            self.free(&mut state, *id, *entry);
        }
        state.generation = state.generation.wrapping_add(1);
        state.page.mark_dirty();
        ids.len()
    }

    /// The advance of the icon at `code_point` in GUI units; `None` for a code point with no icon. Takes no lock.
    pub fn advance_of(&self, code_point: u32) -> Option<f32> {
        let cell = code_point.checked_sub(Self::FIRST_CODE_POINT)? as usize;
        let bits = self.advances.get(cell)?.load(Ordering::Acquire);
        (bits != 0).then(|| f32::from_bits(bits))
    }

    /// Whether any icon is registered. Takes no lock.
    pub fn has_icons(&self) -> bool {
        self.count.load(Ordering::Acquire) != 0
    }

    /// The number of icons.
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    /// Whether the book holds no icon.
    pub fn is_empty(&self) -> bool {
        !self.has_icons()
    }

    /// The pixels of the glyph page when it changed since the last call; the mark is cleared.
    pub fn take_page(&self) -> Option<Vec<u8>> {
        self.lock().page.take_dirty().map(<[u8]>::to_vec)
    }

    /// Marks the page changed again, after an upload that did not happen.
    pub fn mark_page_dirty(&self) {
        self.lock().page.mark_dirty();
    }

    /// Whether the key names an icon of the current generation that was not freed.
    pub fn is_current(&self, key: IconKey) -> bool {
        let state = self.lock();
        key.generation == state.generation && state.icons.contains_key(&key.id)
    }
}
