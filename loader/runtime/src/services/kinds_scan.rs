use std::collections::HashSet;

use crate::structs::{KindsOutcome, KindsScanned};

/// The cells and chunks `KindsScan` reads, by position.
pub trait CellSource {
    /// Whether the chunk at chunk position (`cx`, `cz`) is loaded: its blocks can be read.
    fn chunk_loaded(&mut self, cx: i32, cz: i32) -> bool;

    /// The block type id at `pos`. `None` when the cell cannot be read.
    fn type_at(&mut self, pos: [i32; 3]) -> Option<u32>;
}

/// The distinct block types of a box.
///
/// Every chunk of the box must be loaded before the first cell is read: a chunk that is not gives `NotLoaded` and no
/// cell is read. The cells are then read with x outermost, z next and y innermost, each upward. The kinds come in the
/// order they were first met. Air is a kind like any other. A cell that cannot be read gives `NotLoaded`. More kinds than
/// `max_kinds` gives `TooMany`, at the cell that showed the one too many. A refusal gives no list.
pub struct KindsScan {
    max_kinds: usize,
}

impl KindsScan {
    /// The most cells one box may hold; a caller refuses a larger one.
    pub const CELL_CAP: u64 = 65536;

    // An `i32` shifted by 4 floors to the chunk coordinate, for negative cells too.
    const CHUNK_SHIFT: u32 = 4;

    pub fn new(max_kinds: u32) -> Self {
        Self { max_kinds: max_kinds as usize }
    }

    /// The number of cells of the box from `min` to `max`, both included; `None` when `min` is above `max` on an axis.
    /// A box too large to count gives `u64::MAX`.
    pub fn cells(min: [i32; 3], max: [i32; 3]) -> Option<u64> {
        let mut cells = 1u64;
        for axis in 0..3 {
            let span = i64::from(max[axis]) - i64::from(min[axis]);
            let length = u64::try_from(span).ok()?.saturating_add(1);
            cells = cells.saturating_mul(length);
        }
        Some(cells)
    }

    /// Reads the box from `min` to `max`, both included, through `source`. The box must have at most `u32::MAX` cells
    /// and no axis inverted; the caller checks that with `cells`.
    pub fn scan(&self, min: [i32; 3], max: [i32; 3], source: &mut impl CellSource) -> KindsScanned {
        let refusal = |outcome| KindsScanned { outcome, cells: 0 };
        for cx in (min[0] >> Self::CHUNK_SHIFT)..=(max[0] >> Self::CHUNK_SHIFT) {
            for cz in (min[2] >> Self::CHUNK_SHIFT)..=(max[2] >> Self::CHUNK_SHIFT) {
                if !source.chunk_loaded(cx, cz) {
                    return refusal(KindsOutcome::NotLoaded);
                }
            }
        }
        let mut seen = HashSet::new();
        let mut kinds = Vec::new();
        let mut cells = 0u32;
        for x in min[0]..=max[0] {
            for z in min[2]..=max[2] {
                for y in min[1]..=max[1] {
                    let Some(id) = source.type_at([x, y, z]) else {
                        return refusal(KindsOutcome::NotLoaded);
                    };
                    cells = cells.saturating_add(1);
                    if seen.insert(id) {
                        if kinds.len() >= self.max_kinds {
                            return KindsScanned { outcome: KindsOutcome::TooMany, cells };
                        }
                        kinds.push(id);
                    }
                }
            }
        }
        KindsScanned { outcome: KindsOutcome::Kinds(kinds), cells }
    }
}
