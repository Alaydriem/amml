use amml_abi::{AmmlBlockHandle, AmmlBlockMatch, AmmlItemSnapshot, AmmlStateEntry, AmmlStr, AmmlWorldTable};

use crate::{ApiError, BlockHandle, BlockMatch, BlockTypeId, ItemSnapshot, StateValue};

/// The block types and block states of the game: the names behind the ids that `LevelChunk::read_columns` gives
/// (`Area::WORLD_CHUNK_READ`), and block states as `BlockHandle`s (`Area::WORLD_BLOCK_STATES`).
///
/// A `BlockHandle` holds no table, so the calls that take one are here. They run on the game thread only (a call on
/// another thread gives `None` or `Err(Refused)`), and the handles they give are bound to this mod and the current world.
#[derive(Clone, Copy)]
pub struct Blocks {
    table: Option<AmmlWorldTable>,
    module: &'static str,
}

impl Blocks {
    // A name that is longer than this one is read again with a buffer of its own length.
    const FIRST_READ: usize = 128;

    pub(crate) fn new(table: Option<AmmlWorldTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// The block of the `BlockChanged` event whose `new_state` is `token`, as a handle of this mod; the loader gives it
    /// only inside the listener call of that event.
    pub(crate) fn changed_state(&self, token: u32) -> Option<BlockHandle> {
        let read = self.table?.block_changed_state?;
        let mut out = AmmlBlockHandle::NONE;
        // SAFETY: the table function is the loader's; the module id and the result live for the call.
        #[allow(unsafe_code)]
        let status = unsafe { read(AmmlStr::from_str(self.module), u64::from(token), &raw mut out) };
        if status.is_ok() { BlockHandle::from_raw(out) } else { None }
    }

    /// The full name of the block type `id`, such as `minecraft:stone`. The loader read the name when it made the id,
    /// so this works on any thread and reads no game memory. `None` for `NONE`, an id the loader never gave, a name
    /// that was not readable, and a loader without the area.
    #[allow(unsafe_code)]
    pub fn type_name(&self, id: BlockTypeId) -> Option<String> {
        let read = self.table?.block_type_name?;
        // SAFETY: the table function is the loader's; it writes at most `cap` bytes to the buffer and one length.
        Self::read_text(|buffer, cap, length| unsafe { read(id.0, buffer, cap, length) }.is_ok())
    }

    /// The block state of the block type `name` (such as `minecraft:oak_stairs`) with `states`, as the game reads a
    /// block of a structure. A state the block type does not have, or a value that does not fit it, is left out by the
    /// game: the block then has the default value of that state and `exact` is false. `None` for a name the game does
    /// not know, before a world is joined, off the game thread and without the area. Two calls that give the same
    /// game block give equal handles.
    pub fn from_state(&self, name: &str, states: &[(&str, StateValue<'_>)]) -> Option<BlockMatch> {
        let find = self.table?.block_from_state?;
        let entries: Vec<AmmlStateEntry> = states
            .iter()
            .map(|(key, value)| AmmlStateEntry { name: AmmlStr::from_str(key), value: value.raw() })
            .collect();
        let count = u32::try_from(entries.len()).ok()?;
        let mut found = AmmlBlockMatch { block: AmmlBlockHandle::NONE, exact: 0, _reserved: [0; 7] };
        // SAFETY: the table function is the loader's; the module id, the name, the entries (whose texts borrow from
        // `states`) and the result live for the call.
        #[allow(unsafe_code)]
        let status = unsafe {
            find(AmmlStr::from_str(self.module), AmmlStr::from_str(name), entries.as_ptr(), count, &raw mut found)
        };
        if !status.is_ok() {
            return None;
        }
        BlockHandle::from_raw(found.block).map(|block| BlockMatch { block, exact: found.exact != 0 })
    }

    /// `block` turned by `quarters` clockwise quarter turns seen from above (taken modulo 4), as the game turns a block
    /// of a structure: its rotatable states, then its connection and wall states. A block with no rotatable state
    /// gives an equal handle. `Err(Stale)` for a handle of an earlier world, `Err(Refused)` off the game thread,
    /// `Err(NotReady)` before a world is joined, `Err(NotAvailable)` without the area.
    pub fn rotated(&self, block: BlockHandle, quarters: u8) -> Result<BlockHandle, ApiError> {
        let turn = self.table.and_then(|table| table.block_rotated).ok_or(ApiError::NotAvailable)?;
        let mut out = AmmlBlockHandle::NONE;
        // SAFETY: the table function is the loader's; the module id and the result live for the call.
        #[allow(unsafe_code)]
        let status = unsafe { turn(AmmlStr::from_str(self.module), block.raw(), quarters, &raw mut out) };
        ApiError::check(status)?;
        BlockHandle::from_raw(out).ok_or(ApiError::Loader(amml_abi::AmmlStatus::FAILED))
    }

    /// The full name of the block type of `block`, such as `minecraft:oak_stairs`. `None` for a stale handle, off the
    /// game thread and without the area.
    #[allow(unsafe_code)]
    pub fn name(&self, block: BlockHandle) -> Option<String> {
        let read = self.table?.block_handle_name?;
        // SAFETY: the table function is the loader's; it writes at most `cap` bytes to the buffer and one length.
        Self::read_text(|buffer, cap, length| unsafe { read(block.raw(), buffer, cap, length) }.is_ok())
    }

    /// The item stack the game makes for `block` (as the pick block does), with its type name. `None` for a stale
    /// handle, off the game thread, without the area, and on a build where the loader cannot read item stacks.
    pub fn item(&self, block: BlockHandle) -> Option<ItemSnapshot> {
        let read = self.table?.block_handle_item?;
        let mut raw = AmmlItemSnapshot::empty();
        let mut read_ok = false;
        // The name is read in the same call; a longer name is read again with a buffer of its own length.
        #[allow(unsafe_code)]
        let name = Self::read_text(|buffer, cap, length| {
            // SAFETY: the table function is the loader's; it writes one snapshot of its `size`, at most `cap` name
            // bytes and one length.
            let status = unsafe { read(block.raw(), &raw mut raw, buffer, cap, length) };
            read_ok = status.is_ok();
            read_ok
        });
        read_ok.then(|| ItemSnapshot::from_raw(&raw, name.filter(|name| !name.is_empty())))
    }

    // Reads a text through `read` (buffer, capacity, length out; true on success) into a string, again with a buffer of
    // the full length when the first one was too short. `None` when a read fails or the text is not UTF-8.
    pub(crate) fn read_text(mut read: impl FnMut(*mut u8, usize, *mut usize) -> bool) -> Option<String> {
        let mut buffer = vec![0u8; Self::FIRST_READ];
        let mut length = 0usize;
        if !read(buffer.as_mut_ptr(), buffer.len(), &raw mut length) {
            return None;
        }
        if length > buffer.len() {
            buffer = vec![0u8; length];
            if !read(buffer.as_mut_ptr(), buffer.len(), &raw mut length) || length > buffer.len() {
                return None;
            }
        }
        buffer.truncate(length);
        String::from_utf8(buffer).ok()
    }
}
