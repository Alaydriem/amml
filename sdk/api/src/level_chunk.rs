use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{AmmlChunkLoad, AmmlColumnReadStats, AmmlColumnRequest, AmmlColumnSample, AmmlWorldTable};

use crate::{ChunkLoad, ColumnReadStats, ColumnRequest, ColumnSample};

/// A chunk of the world of the local player (`BlockSource::chunk`), as an opaque handle. It is valid in the game frame
/// of the call that gave it only: a later frame, another dimension or another world gives `None` on every use.
pub struct LevelChunk<'f> {
    handle: *const c_void,
    world: AmmlWorldTable,
    _event: PhantomData<&'f ()>,
}

impl LevelChunk<'_> {
    pub(crate) fn new(handle: *const c_void, world: AmmlWorldTable) -> Self {
        Self { handle, world, _event: PhantomData }
    }

    /// How much of the chunk the client has. `None` when the loader refuses the read (the handle is from an earlier
    /// frame, for example). Area `world.chunk_read`.
    pub fn load_state(&self) -> Option<ChunkLoad> {
        let read = self.world.chunk_load_state?;
        let mut out = AmmlChunkLoad::NOT_LOADED;
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &mut out) };
        status.is_ok().then(|| ChunkLoad::from_abi(out)).flatten()
    }

    /// Reads the columns that `request` asks for into `out`, entry `z * 16 + x` for the column `(x, z)`; the entries
    /// of the other columns are not written. A chunk that is not loaded, or whose top sub-chunk is a placeholder,
    /// marks every asked column `ColumnSample::SKIPPED`. `None` when the loader refuses the read. One call reads a
    /// whole chunk, so ask for the columns of one chunk at a time. Area `world.chunk_read`.
    pub fn read_columns(&self, request: &ColumnRequest, out: &mut [ColumnSample; 256]) -> Option<ColumnReadStats> {
        let read = self.world.read_columns?;
        let abi_request = AmmlColumnRequest::from(request);
        let mut samples = [AmmlColumnSample::from(ColumnSample::default()); 256];
        let mut stats = AmmlColumnReadStats::default();
        // SAFETY: the loader checks the handle before it reads game memory; `abi_request`, `samples` (256 entries)
        // and `stats` are live locals.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &abi_request, samples.as_mut_ptr(), 256, &mut stats) };
        if !status.is_ok() {
            return None;
        }
        for (index, sample) in samples.iter().enumerate() {
            if request.selects_index(index) {
                out[index] = ColumnSample::from(*sample);
            }
        }
        Some(ColumnReadStats::from(stats))
    }
}

impl PartialEq for LevelChunk<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
