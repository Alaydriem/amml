use amml_abi::AmmlWorldTable;

/// The biomes of the game (`Area::WORLD_BIOMES`): the names behind the ids that `LevelChunk::read_columns` gives.
///
/// A biome id is the game's and can belong to one world (a data-driven biome): store the name, not the id.
#[derive(Clone, Copy)]
pub struct Biomes {
    table: Option<AmmlWorldTable>,
}

impl Biomes {
    // A name that is longer than this one is read again with a buffer of its own length.
    const FIRST_READ: usize = 64;

    pub(crate) fn new(table: Option<AmmlWorldTable>) -> Self {
        Self { table }
    }

    /// The name of the biome `id`, such as `plains`. The loader read the name the first time a `read_columns` of the
    /// current world met the id, so this works on any thread and reads no game memory. `None` for `ColumnSample::NO_BIOME`,
    /// an id not met yet in the current world, a name that was not readable, and a loader without the area.
    pub fn name(&self, id: u16) -> Option<String> {
        let read = self.table?.biome_name?;
        let mut buffer = vec![0u8; Self::FIRST_READ];
        let mut length = 0usize;
        // SAFETY: `buffer` holds `cap` bytes and `length` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(id, buffer.as_mut_ptr(), buffer.len(), &mut length) };
        if !status.is_ok() {
            return None;
        }
        if length > buffer.len() {
            buffer = vec![0u8; length];
            // SAFETY: as above, with the buffer of the full length.
            #[allow(unsafe_code)]
            let status = unsafe { read(id, buffer.as_mut_ptr(), buffer.len(), &mut length) };
            if !status.is_ok() || length > buffer.len() {
                return None;
            }
        }
        buffer.truncate(length);
        String::from_utf8(buffer).ok()
    }
}
