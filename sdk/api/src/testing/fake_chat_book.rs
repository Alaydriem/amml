use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};

use amml_abi::{AmmlChatField, AmmlInlineIcon, AmmlStatus};

use crate::testing::{FakeChat, FakeChatEvent, FakeIconData};

// The book of the newest fake. A thread that holds no fake reaches it here: the loader checks a `release` at once on any
// thread and defers only the free, and so does the fake.
static ACTIVE: Mutex<Option<Arc<Mutex<FakeIconData>>>> = Mutex::new(None);

const CELLS: usize = 240;
const FIRST_GLYPH: u32 = 0xE110;
// The longest text one edit takes, as in the loader.
const MAX_EDIT_BYTES: usize = 4096;

/// The chat side of a `FakeGame`: the line of the running event, the inline icons with the loader rules, and the font.
pub(crate) struct FakeChatBook {
    data: Arc<Mutex<FakeIconData>>,
    event: RefCell<Option<FakeChatEvent>>,
}

impl Default for FakeChatBook {
    fn default() -> Self {
        let data = Arc::new(Mutex::new(FakeIconData {
            generation: 0,
            next_id: 1,
            icons: BTreeMap::new(),
            used: vec![false; CELLS],
            released: VecDeque::new(),
            bitmap: false,
            queued: Vec::new(),
        }));
        *ACTIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(data.clone());
        Self { data, event: RefCell::new(None) }
    }
}

impl FakeChatBook {
    fn lock(&self) -> MutexGuard<'_, FakeIconData> {
        self.data.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Whether the default font is a bitmap font.
    pub(crate) fn set_bitmap(&self, bitmap: bool) {
        self.lock().bitmap = bitmap;
    }

    pub(crate) fn bitmap(&self) -> bool {
        self.lock().bitmap
    }

    /// A line is built: the listeners of its event run until `leave`.
    pub(crate) fn enter(&self, chat: &FakeChat) {
        *self.event.borrow_mut() = Some(FakeChatEvent {
            username: chat.username.clone().into(),
            xuid: chat.xuid.clone().into(),
            text: chat.text.clone().into(),
            filtered_text: chat.filtered_text.clone().map(Into::into),
            new_text: None,
            new_filtered_text: None,
        });
    }

    /// The listeners are done: gives the text and the filtered text after the edits.
    pub(crate) fn leave(&self) -> (Option<String>, Option<String>) {
        let Some(event) = self.event.borrow_mut().take() else { return (None, None) };
        let text = event.new_text.unwrap_or_else(|| event.text.into_string());
        let filtered = match (event.new_filtered_text, event.filtered_text) {
            (Some(edit), Some(_)) => Some(edit),
            (_, original) => original.map(str::into_string),
        };
        (Some(text), filtered)
    }

    /// The address and the length of one string of the running line. `REFUSED` when no event runs, `INVALID_ARGUMENT` for
    /// an unknown field, `NOT_AVAILABLE` for the filtered text of a line that has none.
    pub(crate) fn text(&self, field: AmmlChatField) -> Result<(*const u8, usize), AmmlStatus> {
        let event = self.event.borrow();
        let event = event.as_ref().ok_or(AmmlStatus::REFUSED)?;
        let text: &str = match field {
            AmmlChatField::USERNAME => &event.username,
            AmmlChatField::XUID => &event.xuid,
            AmmlChatField::TEXT => &event.text,
            AmmlChatField::FILTERED_TEXT => event.filtered_text.as_deref().ok_or(AmmlStatus::NOT_AVAILABLE)?,
            _ => return Err(AmmlStatus::INVALID_ARGUMENT),
        };
        Ok((text.as_ptr(), text.len()))
    }

    /// An edit of the running line, applied at `leave`. `INVALID_ARGUMENT` for a read only or unknown field, `REFUSED` for
    /// a text over 4096 bytes and when no event runs, `NOT_AVAILABLE` for the filtered text of a line that has none.
    pub(crate) fn set_text(&self, field: AmmlChatField, text: &str) -> Result<(), AmmlStatus> {
        if field != AmmlChatField::TEXT && field != AmmlChatField::FILTERED_TEXT {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if text.len() > MAX_EDIT_BYTES {
            return Err(AmmlStatus::REFUSED);
        }
        let mut event = self.event.borrow_mut();
        let event = event.as_mut().ok_or(AmmlStatus::REFUSED)?;
        if field == AmmlChatField::TEXT {
            event.new_text = Some(text.to_string());
        } else if event.filtered_text.is_some() {
            event.new_filtered_text = Some(text.to_string());
        } else {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(())
    }

    /// Registers an icon; the cell is the lowest unused one, else the one released first. `NOT_AVAILABLE` without a bitmap
    /// font, `REFUSED` when all cells are in use.
    pub(crate) fn register(&self) -> Result<AmmlInlineIcon, AmmlStatus> {
        let mut data = self.lock();
        if !data.bitmap {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let cell = match data.used.iter().position(|used| !used) {
            Some(cell) => cell,
            None => data.released.pop_front().ok_or(AmmlStatus::REFUSED)?,
        };
        data.used[cell] = true;
        let id = data.next_id;
        data.next_id += 1;
        data.icons.insert(id, cell);
        Ok(AmmlInlineIcon { id, generation: data.generation, glyph: FIRST_GLYPH + cell as u32 })
    }

    /// `STALE` for an icon that is not current.
    pub(crate) fn check(&self, icon: AmmlInlineIcon) -> Result<(), AmmlStatus> {
        self.lock().current(icon).map(|_| ())
    }

    /// Frees `icon` at once; `STALE` for one that is not current.
    pub(crate) fn release(&self, icon: AmmlInlineIcon) -> Result<(), AmmlStatus> {
        let mut data = self.lock();
        data.take(icon)?;
        data.free(icon);
        Ok(())
    }

    /// The release of `icon` on a thread that holds no fake: checked at once (`STALE` for an icon that is not current,
    /// also one already queued), freed at `pump`.
    pub(crate) fn release_off_thread(icon: AmmlInlineIcon) -> Result<(), AmmlStatus> {
        let active = ACTIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone();
        let Some(active) = active else { return Err(AmmlStatus::FAILED) };
        let mut data = active.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        data.take(icon)?;
        // The queued icon cannot be used again, but its cell is not free until the pump.
        data.icons.remove(&icon.id);
        data.queued.push(icon);
        Ok(())
    }

    /// Applies the queued releases; gives the number of icons freed.
    pub(crate) fn pump(&self) -> usize {
        let mut data = self.lock();
        let queued = std::mem::take(&mut data.queued);
        let count = queued.len();
        for icon in queued {
            let cell = (icon.glyph - FIRST_GLYPH) as usize;
            data.released.push_back(cell);
        }
        count
    }

    /// Every icon is freed and the generation goes up.
    pub(crate) fn leave_world(&self) {
        let mut data = self.lock();
        let cells: Vec<usize> = data.icons.values().copied().collect();
        data.icons.clear();
        data.released.extend(cells);
        data.generation += 1;
    }

    /// The characters of the icons that are not freed yet: the live ones and the ones queued for a free, in cell order.
    pub(crate) fn glyphs(&self) -> Vec<char> {
        let data = self.lock();
        let mut cells: Vec<usize> = data.icons.values().copied().collect();
        cells.extend(data.queued.iter().map(|icon| (icon.glyph - FIRST_GLYPH) as usize));
        cells.sort_unstable();
        cells.into_iter().filter_map(|cell| char::from_u32(FIRST_GLYPH + cell as u32)).collect()
    }
}
