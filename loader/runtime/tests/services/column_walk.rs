use std::collections::HashMap;

use amml_runtime::{ColumnBlocks, ColumnWalk, WalkBlock, WalkedColumn};

const STONE: u32 = 1;
const WATER: u32 = 2;
const PLACEHOLDER: u32 = 3;
const PLANT: u32 = 4;

// One column of blocks by y; every y not set is air. Records each y the walk asked for.
struct Column {
    blocks: HashMap<i32, WalkBlock>,
    asked: Vec<i32>,
}

impl Column {
    fn new() -> Self {
        Self { blocks: HashMap::new(), asked: Vec::new() }
    }

    fn fill(mut self, from: i32, to: i32, block: WalkBlock) -> Self {
        for y in from..=to {
            self.blocks.insert(y, block);
        }
        self
    }
}

impl ColumnBlocks for Column {
    fn block(&mut self, y: i32) -> Option<WalkBlock> {
        self.asked.push(y);
        self.blocks.get(&y).copied()
    }
}

fn solid(id: u32) -> WalkBlock {
    WalkBlock { id, is_air: false, is_water: false, is_placeholder: false, is_solid: true }
}

fn water() -> WalkBlock {
    WalkBlock { id: WATER, is_air: false, is_water: true, is_placeholder: false, is_solid: false }
}

fn plant() -> WalkBlock {
    WalkBlock { id: PLANT, is_air: false, is_water: false, is_placeholder: false, is_solid: false }
}

fn placeholder() -> WalkBlock {
    WalkBlock { id: PLACEHOLDER, is_air: false, is_water: false, is_placeholder: true, is_solid: true }
}

fn shows(column: &WalkedColumn, id: u32, y: i32) -> bool {
    column.shown == id && column.shown_y == y
}

#[test]
fn surface_under_camera_height() {
    let walk = ColumnWalk::new(-4, 12, false);
    let mut blocks = Column::new().fill(-4, 5, solid(STONE));

    let from_air = walk.walk(&mut blocks, 9, 5);
    let from_above = walk.walk(&mut blocks, 40, 5);

    assert!(shows(&from_air, STONE, 5));
    assert!(shows(&from_above, STONE, 5));
    assert!(!from_air.void && !from_air.retry);
    assert_eq!(from_air.water, 0);
}

#[test]
fn camera_below_the_chunk_shows_nothing_and_all_air_is_void() {
    let walk = ColumnWalk::new(-4, 12, false);
    let mut stone = Column::new().fill(-4, 5, solid(STONE));
    let mut empty = Column::new();

    assert_eq!(walk.walk(&mut stone, -9, 5), WalkedColumn::default());
    let void = walk.walk(&mut empty, 9, -4);

    assert!(void.void);
    assert_eq!(void.shown, 0);
}

#[test]
fn transparent_water_gives_depth_and_floor() {
    let transparent = ColumnWalk::new(-4, 12, true);
    let opaque = ColumnWalk::new(-4, 12, false);
    let mut blocks = Column::new().fill(-4, 2, solid(STONE)).fill(3, 5, water());

    let clear = transparent.walk(&mut blocks, 9, 5);
    let seen = opaque.walk(&mut blocks, 9, 5);

    assert!(shows(&clear, STONE, 2));
    assert_eq!((clear.water, clear.water_depth), (WATER, 3));
    assert!(shows(&seen, WATER, 5));
    assert_eq!((seen.water, seen.water_depth), (0, 0));
}

#[test]
fn water_down_to_the_bottom_shows_the_water_with_the_depth_to_the_bottom() {
    let walk = ColumnWalk::new(-4, 12, true);
    let mut blocks = Column::new().fill(-4, 5, water());

    let column = walk.walk(&mut blocks, 9, 5);

    assert!(shows(&column, WATER, 5));
    assert_eq!((column.water, column.water_depth), (WATER, 9));
}

#[test]
fn cave_mode_walks_the_under_solid_run() {
    let walk = ColumnWalk::new(-4, 12, false);
    // A cave of two air blocks between stone, and the camera in the stone above it.
    let mut cave = Column::new().fill(-4, 2, solid(STONE)).fill(5, 10, solid(STONE));
    // A plant in the solid run: the first block that is not solid is shown.
    let mut run = Column::new().fill(-4, 10, solid(STONE)).fill(6, 6, plant());

    let under_air = walk.walk(&mut cave, 8, 10);
    let at_plant = walk.walk(&mut run, 8, 10);

    assert!(shows(&under_air, STONE, 2));
    assert!(shows(&at_plant, PLANT, 6));
}

#[test]
fn cave_mode_with_air_to_the_bottom_is_void_and_a_solid_run_shows_nothing() {
    let walk = ColumnWalk::new(-4, 12, false);
    let mut hollow = Column::new().fill(5, 10, solid(STONE));
    let mut full = Column::new().fill(-4, 10, solid(STONE));

    assert!(walk.walk(&mut hollow, 8, 10).void);
    assert_eq!(walk.walk(&mut full, 8, 10), WalkedColumn::default());
}

#[test]
fn camera_in_a_block_that_is_not_solid_shows_it() {
    let walk = ColumnWalk::new(-4, 12, false);
    let mut blocks = Column::new().fill(-4, 5, solid(STONE)).fill(6, 6, plant());

    assert!(shows(&walk.walk(&mut blocks, 6, 6), PLANT, 6));
}

#[test]
fn placeholder_block_sets_retry() {
    let walk = ColumnWalk::new(-4, 12, false);
    let mut blocks = Column::new().fill(-4, 4, solid(STONE)).fill(5, 5, placeholder());

    let column = walk.walk(&mut blocks, 9, 5);

    assert!(column.retry);
    assert!(shows(&column, PLACEHOLDER, 5));
}

#[test]
fn walk_stays_inside_min_and_max_y() {
    let walk = ColumnWalk::new(-4, 12, true);
    let cases = [(100, 5), (100, 99), (-100, 5), (9, -50), (9, 100), (11, 5), (12, 5), (-4, -4)];
    for (camera, height) in cases {
        let mut empty = Column::new();
        let mut stone = Column::new().fill(-4, 5, solid(STONE));
        let mut wet = Column::new().fill(-4, 11, water());

        walk.walk(&mut empty, camera, height);
        walk.walk(&mut stone, camera, height);
        walk.walk(&mut wet, camera, height);

        for blocks in [&empty, &stone, &wet] {
            assert!(blocks.asked.iter().all(|y| (-4..12).contains(y)), "{camera} {height}: {:?}", blocks.asked);
        }
    }
}
