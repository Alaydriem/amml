use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Instant;

use amml_abi::{AmmlBlockPos, AmmlKindsResult, AmmlStatus, AmmlThread};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::{BlockTypeBook, CellSource, KindsOutcome, KindsScan};

use crate::{BlockReadAdapter, BlockTypeReader, ChunkAdapter, LoaderState, ThreadNote, WorldAdapter};

static ADAPTER: OnceLock<KindsAdapter> = OnceLock::new();

/// The `world.kinds_in_box` area: the distinct block types of a box of the block source of the local player.
///
/// The call checks the gate, the game thread, the block source and the box (no axis inverted, at most
/// `KindsScan::CELL_CAP` cells; `REFUSED` otherwise), then `KindsScan` asks that every chunk of the box is loaded
/// (`ChunkAdapter::is_loaded`, `BlockSource::getChunk`) before it reads a cell. A cell is read with
/// `BlockSource::getBlock` and its block type is given an id by the `BlockTypeReader` of the chunk reads, so the ids are the
/// ones of `world.chunk_read` and `block_type_name` names them. A block with no readable type, and a type with no readable
/// name, answers `NOT_LOADED`.
///
/// Every call runs on the game thread only. The first call that reads cells, and each later one that is slower than
/// 1 ms and slower than all before it, logs `kinds scan time` with the cells and the microseconds.
pub struct KindsAdapter {
    state: &'static LoaderState,
    thread: ThreadNote,
    logged: AtomicBool,
    slowest_us: AtomicU32,
}

impl KindsAdapter {
    pub const AREA: &'static str = "world.kinds_in_box";
    const VERSION: u32 = 1;
    // A scan that takes longer than this is logged when it is the slowest so far.
    const SLOW_US: u32 = 1000;

    /// Offers `world.kinds_in_box` when its catalog shape resolved and the chunk reads and the block reads are installed,
    /// and logs one `area adapter` or `area not available` line. Works once per process: a second call offers nothing and
    /// gives the catalog row. Install after `WorldAdapter` and `BlockReadAdapter`.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id == Self::AREA)
            .collect();
        for row in &mut rows {
            if row.shape.is_none() {
                curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") });
                return rows;
            }
            let chunks = WorldAdapter::get().is_some_and(|world| world.chunks().is_some());
            if !chunks || BlockReadAdapter::get().is_none() {
                row.shape = None;
                row.missing = vec!["chunk adapter", "block read adapter"];
                curia::info!("area not available", { area: row.id, missing: "chunk adapter, block read adapter" });
                return rows;
            }
        }
        let adapter =
            Self { state, thread: ThreadNote::new(), logged: AtomicBool::new(false), slowest_us: AtomicU32::new(0) };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the kinds adapter is already installed");
            return rows;
        }
        state.areas().offer(Self::AREA, Self::VERSION);
        curia::info!("area adapter", { area: Self::AREA, shape: "default" });
        rows
    }

    /// The installed adapter; `None` before `install` or when the area was not offered.
    pub fn get() -> Option<&'static KindsAdapter> {
        ADAPTER.get()
    }

    /// Lists the distinct block type ids of the box from `min` to `max` (both included) of `source`, a block source handle
    /// of the current world, in `ids` (up to `ids.len()` of them), and describes the answer. `REFUSED` for an inverted
    /// box, a box of more than `KindsScan::CELL_CAP` cells, a block source that is not the current one and off the game thread.
    pub fn kinds_in_box(
        &self,
        source: *const c_void,
        min: AmmlBlockPos,
        max: AmmlBlockPos,
        ids: &mut [u32],
    ) -> Result<AmmlKindsResult, AmmlStatus> {
        self.enter()?;
        let world = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let source = world.block_source(Self::AREA, Self::VERSION, source)?;
        let (min, max) = ([min.x, min.y, min.z], [max.x, max.y, max.z]);
        match KindsScan::cells(min, max) {
            Some(cells) if cells <= KindsScan::CELL_CAP => {}
            _ => return Err(AmmlStatus::REFUSED),
        }
        let chunks = world.chunks().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let reads = BlockReadAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let types = world.block_types().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let mut cells = Cells {
            chunks,
            reads,
            types,
            source,
            epoch: self.state.events().gate().epoch().0,
            known: HashMap::new(),
            last: None,
        };
        let started = Instant::now();
        let scanned = KindsScan::new(u32::try_from(ids.len()).unwrap_or(u32::MAX)).scan(min, max, &mut cells);
        let elapsed_us = u32::try_from(started.elapsed().as_micros()).unwrap_or(u32::MAX);
        if scanned.cells > 0 {
            self.note_time(scanned.cells, elapsed_us);
        }
        let mut result = AmmlKindsResult {
            kind: AmmlKindsResult::NOT_LOADED,
            cells: scanned.cells,
            elapsed_us,
            ..Default::default()
        };
        match scanned.outcome {
            KindsOutcome::Kinds(found) => {
                // A type whose name was not readable stops the scan: a mod could not name its id.
                if found.iter().any(|id| types.type_name(*id).is_none()) {
                    return Ok(result);
                }
                for (slot, id) in ids.iter_mut().zip(&found) {
                    *slot = *id;
                }
                result.kind = AmmlKindsResult::KINDS;
                result.count = u32::try_from(found.len()).map_err(|_| AmmlStatus::FAILED)?;
            }
            KindsOutcome::NotLoaded => {}
            KindsOutcome::TooMany => result.kind = AmmlKindsResult::TOO_MANY,
        }
        Ok(result)
    }

    // The gate, the area and the game thread; logs the thread of the calls once and on a change.
    fn enter(&self) -> Result<(), AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(Self::AREA, Self::VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let on_game_thread = gate.on_game_thread();
        let thread = if on_game_thread { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.thread.changed(thread) {
            curia::info!("world thread", { hook: "BlockSource.getBlock", game: on_game_thread });
        }
        if on_game_thread { Ok(()) } else { Err(AmmlStatus::REFUSED) }
    }

    // Logs the scan time of the first scan, and of each later one that is slower than `SLOW_US` and than all before it.
    fn note_time(&self, cells: u32, us: u32) {
        let first = !self.logged.swap(true, Ordering::AcqRel);
        let slowest = self.slowest_us.fetch_max(us, Ordering::AcqRel);
        if first || (us > Self::SLOW_US && us > slowest) {
            curia::info!("kinds scan time", { cells: cells, us: us });
        }
    }
}

// The cells and chunks of the live block source `source`, for one scan on the game thread.
struct Cells<'a> {
    chunks: &'a ChunkAdapter,
    reads: &'a BlockReadAdapter,
    types: &'a BlockTypeReader,
    source: usize,
    epoch: u64,
    // The type id of each game block met in this scan, by the block's address.
    known: HashMap<usize, u32>,
    // The block of the last cell and its id, so a run of one block is looked up once.
    last: Option<(usize, u32)>,
}

impl CellSource for Cells<'_> {
    fn chunk_loaded(&mut self, cx: i32, cz: i32) -> bool {
        // SAFETY: `source` is the live block source of the local player of the primary client (checked by the caller),
        // on the game thread.
        unsafe { self.chunks.is_loaded(self.source, cx, cz) }.unwrap_or(false)
    }

    fn type_at(&mut self, pos: [i32; 3]) -> Option<u32> {
        // SAFETY: as above; `pos` is a live position.
        let block = unsafe { self.reads.block_address(self.source, pos) };
        if block == 0 {
            return None;
        }
        if let Some((last, id)) = self.last
            && last == block
        {
            return Some(id);
        }
        let id = match self.known.get(&block) {
            Some(id) => *id,
            None => {
                // SAFETY: `block` is the live game block the call just gave, on the game thread.
                let shape = unsafe { self.types.walk_block(block, self.epoch) }?;
                self.known.insert(block, shape.id);
                shape.id
            }
        };
        if id == BlockTypeBook::NONE {
            return None;
        }
        self.last = Some((block, id));
        Some(id)
    }
}
