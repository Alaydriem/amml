use std::collections::{HashMap, HashSet};

use amml_runtime::{CellSource, KindsOutcome, KindsScan};

const AIR: u32 = 1;
const STONE: u32 = 2;
const DIRT: u32 = 3;
const GLASS: u32 = 4;

// A world of chunks that are loaded and blocks by position; a position not set is air. Records every chunk asked
// for and every cell read.
struct World {
    loaded: HashSet<(i32, i32)>,
    blocks: HashMap<[i32; 3], u32>,
    chunk_asks: Vec<(i32, i32)>,
    cell_reads: Vec<[i32; 3]>,
}

impl World {
    fn loaded(chunks: &[(i32, i32)]) -> Self {
        Self {
            loaded: chunks.iter().copied().collect(),
            blocks: HashMap::new(),
            chunk_asks: Vec::new(),
            cell_reads: Vec::new(),
        }
    }

    fn with(mut self, pos: [i32; 3], id: u32) -> Self {
        self.blocks.insert(pos, id);
        self
    }
}

impl CellSource for World {
    fn chunk_loaded(&mut self, cx: i32, cz: i32) -> bool {
        self.chunk_asks.push((cx, cz));
        self.loaded.contains(&(cx, cz))
    }

    fn type_at(&mut self, pos: [i32; 3]) -> Option<u32> {
        self.cell_reads.push(pos);
        Some(self.blocks.get(&pos).copied().unwrap_or(AIR))
    }
}

#[test]
fn unloaded_chunk_gives_not_loaded() {
    // The box covers chunks (0, 0) and (1, 0); the second one is not loaded.
    let mut world = World::loaded(&[(0, 0)]).with([0, 0, 0], STONE);

    let scanned = KindsScan::new(16).scan([0, 0, 0], [20, 3, 3], &mut world);

    assert_eq!(scanned.outcome, KindsOutcome::NotLoaded);
    assert_eq!(scanned.cells, 0);
    assert!(world.cell_reads.is_empty());
    assert!(world.chunk_asks.contains(&(0, 0)) && world.chunk_asks.contains(&(1, 0)));
}

#[test]
fn negative_positions_ask_the_chunk_below_zero() {
    let mut world = World::loaded(&[(-1, -1)]);

    let scanned = KindsScan::new(16).scan([-1, 0, -1], [-1, 0, -1], &mut world);

    assert_eq!(scanned.outcome, KindsOutcome::Kinds(vec![AIR]));
    assert_eq!(world.chunk_asks, [(-1, -1)]);
}

#[test]
fn more_kinds_than_the_cap_gives_too_many() {
    let mut world = World::loaded(&[(0, 0)]).with([0, 0, 0], STONE).with([0, 1, 0], DIRT).with([0, 2, 0], GLASS);

    // The column holds stone, dirt, glass and air above them: four kinds.
    let over = KindsScan::new(3).scan([0, 0, 0], [0, 3, 0], &mut world);
    let exact = KindsScan::new(4).scan(
        [0, 0, 0],
        [0, 3, 0],
        &mut World::loaded(&[(0, 0)]).with([0, 0, 0], STONE).with([0, 1, 0], DIRT).with([0, 2, 0], GLASS),
    );

    assert_eq!(over.outcome, KindsOutcome::TooMany);
    assert_eq!(exact.outcome, KindsOutcome::Kinds(vec![STONE, DIRT, GLASS, AIR]));
}

#[test]
fn kinds_keep_first_seen_order() {
    // The scan reads x, then z, then y; air counts as a kind.
    let mut world = World::loaded(&[(0, 0)])
        .with([0, 0, 0], GLASS)
        .with([0, 1, 0], STONE)
        .with([1, 0, 0], DIRT)
        .with([1, 1, 0], STONE);

    let scanned = KindsScan::new(16).scan([0, 0, 0], [1, 2, 0], &mut world);

    assert_eq!(scanned.outcome, KindsOutcome::Kinds(vec![GLASS, STONE, AIR, DIRT]));
    assert_eq!(scanned.cells, 6);
}

#[test]
fn a_cell_that_cannot_be_read_gives_not_loaded() {
    struct Hole;
    impl CellSource for Hole {
        fn chunk_loaded(&mut self, _: i32, _: i32) -> bool {
            true
        }
        fn type_at(&mut self, _: [i32; 3]) -> Option<u32> {
            None
        }
    }

    assert_eq!(KindsScan::new(16).scan([0, 0, 0], [1, 1, 1], &mut Hole).outcome, KindsOutcome::NotLoaded);
}

#[test]
fn cells_counts_the_box_and_refuses_an_inverted_one() {
    assert_eq!(KindsScan::cells([0, 0, 0], [0, 0, 0]), Some(1));
    assert_eq!(KindsScan::cells([-2, 5, 0], [1, 6, 9]), Some(4 * 2 * 10));
    assert_eq!(KindsScan::cells([0, 1, 0], [0, 0, 0]), None);
    assert_eq!(KindsScan::cells([i32::MIN, i32::MIN, i32::MIN], [i32::MAX, i32::MAX, i32::MAX]), Some(u64::MAX));
}
