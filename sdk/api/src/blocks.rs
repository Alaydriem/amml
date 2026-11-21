use amml_abi::AmmlWorldTable;

use crate::BlockTypeId;

/// The block types of the game (`Area::WORLD_CHUNK_READ`): the names behind the ids that `LevelChunk::read_columns`
/// gives.
#[derive(Clone, Copy)]
pub struct Blocks {
    table: Option<AmmlWorldTable>,
}

impl Blocks {
    // A name that is longer than this one is read again with a buffer of its own length.
    const FIRST_READ: usize = 128;

    pub(crate) fn new(table: Option<AmmlWorldTable>) -> Self {
        Self { table }
    }

    /// The full name of the block type `id`, such as `minecraft:stone`. The loader read the name when it made the id,
    /// so this works on any thread and reads no game memory. `None` for `NONE`, an id the loader never gave, a name
    /// that was not readable, and a loader without the area.
    pub fn type_name(&self, id: BlockTypeId) -> Option<String> {
        let read = self.table?.block_type_name?;
        let mut buffer = vec![0u8; Self::FIRST_READ];
        let mut length = 0usize;
        // SAFETY: `buffer` holds `cap` bytes and `length` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(id.0, buffer.as_mut_ptr(), buffer.len(), &mut length) };
        if !status.is_ok() {
            return None;
        }
        if length > buffer.len() {
            buffer = vec![0u8; length];
            // SAFETY: as above, with the buffer of the full length.
            #[allow(unsafe_code)]
            let status = unsafe { read(id.0, buffer.as_mut_ptr(), buffer.len(), &mut length) };
            if !status.is_ok() || length > buffer.len() {
                return None;
            }
        }
        buffer.truncate(length);
        String::from_utf8(buffer).ok()
    }
}
