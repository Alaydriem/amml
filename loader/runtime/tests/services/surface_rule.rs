use std::collections::HashMap;

use amml_runtime::{SurfaceColumn, SurfaceHit, SurfaceRule};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Air,
    Stone,
    Flower,
    Slab,
    Unreadable,
}

// One column of blocks by y; every y not set is air. Light is `(sky, block)` by y, and records each y asked for.
struct Column {
    blocks: HashMap<i32, Kind>,
    lights: HashMap<i32, (u8, u8)>,
    block_reads: Vec<i32>,
}

impl Column {
    fn new() -> Self {
        Self { blocks: HashMap::new(), lights: HashMap::new(), block_reads: Vec::new() }
    }

    fn fill(mut self, from: i32, to: i32, kind: Kind) -> Self {
        for y in from..=to {
            self.blocks.insert(y, kind);
        }
        self
    }

    fn lit(mut self, y: i32, sky: u8, block: u8) -> Self {
        self.lights.insert(y, (sky, block));
        self
    }
}

impl SurfaceColumn for Column {
    type Block = Kind;

    fn block(&mut self, y: i32) -> Option<Kind> {
        self.block_reads.push(y);
        let kind = self.blocks.get(&y).copied().unwrap_or(Kind::Air);
        (kind != Kind::Unreadable).then_some(kind)
    }

    fn is_air(&mut self, block: Kind) -> bool {
        block == Kind::Air
    }

    fn has_empty_collision(&mut self, block: Kind, _y: i32) -> bool {
        block == Kind::Flower
    }

    fn gives_full_top_support(&mut self, block: Kind) -> bool {
        block == Kind::Stone
    }

    fn light(&mut self, y: i32) -> Option<(u8, u8)> {
        Some(self.lights.get(&y).copied().unwrap_or((0, 0)))
    }
}

fn scan(rule: &SurfaceRule, column: &mut Column, y_min: i32, y_max: i32, room: usize) -> Option<Vec<SurfaceHit>> {
    let mut out = Vec::new();
    let count = rule.scan(column, y_min, y_max, room, |hit| out.push(hit))?;
    assert_eq!(count, out.len());
    Some(out)
}

fn ys(hits: &[SurfaceHit]) -> Vec<i32> {
    hits.iter().map(|hit| hit.y).collect()
}

#[test]
fn full_top_with_air_above_is_a_surface() {
    let rule = SurfaceRule::new(0, 15);
    let mut column = Column::new().fill(0, 5, Kind::Stone);

    let hits = scan(&rule, &mut column, 0, 15, 8).unwrap();

    assert_eq!(ys(&hits), [5]);
}

#[test]
fn a_block_that_does_not_give_full_support_is_no_surface() {
    let rule = SurfaceRule::new(0, 15);
    let mut column = Column::new().fill(0, 3, Kind::Stone).fill(4, 4, Kind::Slab);

    let hits = scan(&rule, &mut column, 0, 15, 8).unwrap();

    assert!(hits.is_empty());
}

#[test]
fn flower_above_counts_as_passable() {
    let rule = SurfaceRule::new(0, 15);
    let mut column = Column::new().fill(0, 4, Kind::Stone).fill(5, 5, Kind::Flower);

    let hits = scan(&rule, &mut column, 0, 15, 8).unwrap();

    assert_eq!(ys(&hits), [4]);
}

#[test]
fn light_comes_from_the_block_above() {
    let rule = SurfaceRule::new(0, 15);
    let mut column = Column::new().fill(0, 4, Kind::Stone).lit(4, 1, 2).lit(5, 13, 6);

    let hits = scan(&rule, &mut column, 0, 15, 8).unwrap();

    assert_eq!(hits, [SurfaceHit { y: 4, sky_light: 13, block_light: 6 }]);
}

#[test]
fn clamped_to_chunk_height() {
    // Blocks from y -4 to y 10 (the highest block y of the chunk). The range asked goes far outside on both sides.
    let rule = SurfaceRule::new(-4, 10);
    let mut column = Column::new().fill(-4, 9, Kind::Stone);

    let hits = scan(&rule, &mut column, -100, 100, 32).unwrap();

    // The lowest block read is the chunk bottom; the highest is the chunk top, which is the block above the highest
    // possible surface (y 9).
    assert_eq!(column.block_reads.iter().min().copied(), Some(-4));
    assert_eq!(column.block_reads.iter().max().copied(), Some(10));
    assert_eq!(ys(&hits), [9]);
}

#[test]
fn a_support_block_at_the_top_of_the_chunk_has_no_block_above_to_read() {
    let rule = SurfaceRule::new(0, 15);
    let mut column = Column::new().fill(0, 15, Kind::Stone);

    let hits = scan(&rule, &mut column, 0, 15, 8).unwrap();

    assert!(hits.is_empty());
    assert_eq!(column.block_reads.iter().max().copied(), Some(15));
}

#[test]
fn a_range_outside_the_chunk_reads_nothing() {
    let rule = SurfaceRule::new(0, 15);
    let mut column = Column::new().fill(0, 5, Kind::Stone);

    let hits = scan(&rule, &mut column, 40, 80, 8).unwrap();

    assert!(hits.is_empty());
    assert!(column.block_reads.is_empty());
}

#[test]
fn stops_at_out_len() {
    let rule = SurfaceRule::new(0, 20);
    // Stone at 2, 6 and 10 with air between: three surfaces, room for two.
    let mut column = Column::new().fill(2, 2, Kind::Stone).fill(6, 6, Kind::Stone).fill(10, 10, Kind::Stone);

    let hits = scan(&rule, &mut column, 0, 20, 2).unwrap();

    assert_eq!(ys(&hits), [2, 6]);
    assert!(!column.block_reads.contains(&11));
}

#[test]
fn surfaces_come_lowest_first() {
    let rule = SurfaceRule::new(0, 20);
    let mut column = Column::new().fill(2, 2, Kind::Stone).fill(6, 6, Kind::Stone).fill(10, 10, Kind::Stone);

    let hits = scan(&rule, &mut column, 0, 20, 8).unwrap();

    assert_eq!(ys(&hits), [2, 6, 10]);
}

#[test]
fn a_block_that_cannot_be_read_gives_none() {
    let rule = SurfaceRule::new(0, 15);
    let mut column = Column::new().fill(0, 3, Kind::Stone).fill(4, 4, Kind::Unreadable);

    assert!(scan(&rule, &mut column, 0, 15, 8).is_none());
}
