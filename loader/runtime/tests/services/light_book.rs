use amml_abi::AmmlLightFalloff;
use amml_runtime::{LightBook, LightPoint, LightRefusal};

fn light(x: f32, level: u8) -> Option<LightPoint> {
    LightPoint::at([x, 64.5, 0.5], level, AmmlLightFalloff::ROUND, 0)
}

fn quiet(_: &[LightPoint]) {}

#[test]
fn a_new_light_marks_the_subchunks_it_lights() {
    let book = LightBook::new();
    let change = book.set(1, light(0.5, 3), quiet).unwrap();
    // Lit blocks reach x and z -2..2, y 62..66; with the one-block border that is subchunks -1..0, 3..4, -1..0.
    assert_eq!(change.marks(), 8);
    assert!(change.contains([0, 4, 0]) && change.contains([-1, 3, -1]));
}

#[test]
fn a_move_inside_a_subchunk_marks_only_that_subchunk() {
    let book = LightBook::new();
    book.set(1, LightPoint::at([8.5, 72.5, 8.5], 3, AmmlLightFalloff::ROUND, 0), quiet).unwrap();
    let change = book.set(1, LightPoint::at([8.75, 72.5, 8.5], 3, AmmlLightFalloff::ROUND, 0), quiet).unwrap();
    assert_eq!(change.marks(), 1);
    assert_eq!(change.boxes(), vec![([0, 64, 0], [15, 79, 15])]);
}

#[test]
fn the_same_light_changes_nothing_and_publishes_nothing() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    let mut published = false;
    let change = book.set(1, light(0.6, 3), |_| published = true).unwrap();
    assert!(change.is_empty());
    assert!(!published);
}

#[test]
fn a_falloff_change_rebuilds_without_a_move() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    let taxicab = LightPoint::at([0.5, 64.5, 0.5], 3, AmmlLightFalloff::TAXICAB, 0);
    assert!(!book.set(1, taxicab, quiet).unwrap().is_empty());
}

#[test]
fn clear_rebuilds_the_old_reach() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    let mut seen = None;
    let change = book.set(1, None, |points| seen = Some(points.len())).unwrap();
    assert_eq!(change.marks(), 8);
    assert_eq!(seen, Some(0));
    assert!(!book.holds(1));
    assert!(book.set(1, None, quiet).unwrap().is_empty());
}

#[test]
fn fifth_slot_is_refused() {
    let book = LightBook::new();
    for owner in 0..4 {
        book.set(owner, light(owner as f32, 3), quiet).unwrap();
    }
    assert_eq!(book.set(9, light(0.5, 3), quiet), Err(LightRefusal::Full));
    assert_eq!(book.points().len(), 4);
    assert!(book.set(2, light(40.0, 5), quiet).is_ok());
}

#[test]
fn disable_one_of_two_keeps_the_other() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    book.set(2, light(100.5, 5), quiet).unwrap();
    let change = book.set(1, None, quiet).unwrap();
    assert_eq!(change.marks(), 8);
    assert!(!change.contains([6, 4, 0]));
    assert_eq!(book.points(), vec![light(100.5, 5).unwrap()]);
}

#[test]
fn clear_all_rebuilds_every_reach() {
    let book = LightBook::new();
    book.set(1, light(0.5, 3), quiet).unwrap();
    book.set(2, light(100.5, 5), quiet).unwrap();
    let change = book.clear_all(quiet);
    assert!(change.contains([0, 4, 0]) && change.contains([6, 4, 0]));
    assert!(book.points().is_empty());
}

// The sub-chunks a block box covers (inclusive block min, max).
fn subchunks_of(found: &([i32; 3], [i32; 3])) -> Vec<[i32; 3]> {
    let (low, high) = found;
    let range = |axis: usize| low[axis].div_euclid(16)..=high[axis].div_euclid(16);
    let mut out = Vec::new();
    for x in range(0) {
        for y in range(1) {
            for z in range(2) {
                out.push([x, y, z]);
            }
        }
    }
    out
}

// A walk at 4.3 blocks per second, one light update per 20 Hz tick, for 10 s, across sub-chunk borders.
fn walk(level: u8) -> Vec<Option<LightPoint>> {
    let direction = [0.96f32, 0.0, 0.28];
    let per_tick = 4.3f32 / 20.0;
    (0..200)
        .map(|tick| {
            let along = per_tick * tick as f32;
            let position = [-10.3 + direction[0] * along, 70.62, 5.1 + direction[2] * along];
            LightPoint::at(position, level, AmmlLightFalloff::ROUND, 0)
        })
        .collect()
}

// Changes, the sub-chunk marks the game's `_setDirty` makes for them (a box marks each sub-chunk it covers) and
// the distinct sub-chunks per change.
fn count_walk(level: u8) -> (usize, usize, usize, u64) {
    let book = LightBook::new();
    let (mut changes, mut marks, mut distinct, mut evaluations) = (0, 0, 0, 0);
    for point in walk(level) {
        let change = book.set(1, point, quiet).unwrap();
        if change.is_empty() {
            continue;
        }
        changes += 1;
        let covered: Vec<[i32; 3]> = change.boxes().iter().flat_map(subchunks_of).collect();
        marks += covered.len();
        evaluations += change.evaluations();
        let mut unique = covered.clone();
        unique.sort_unstable();
        unique.dedup();
        distinct += unique.len();
    }
    (changes, marks, distinct, evaluations)
}

// Before the diff (old and new reach boxes): level 15 gave 5562 marks (4347 distinct) and level 7 1528 (1200) for
// the 175 changes of this walk. The limits are the measured values after it; raise them with a new measurement when
// a correct change needs more marks.
#[test]
fn a_walk_marks_each_subchunk_once_and_only_where_light_differs() {
    for (level, limit) in [(15u8, 3570usize), (7, 799)] {
        let (changes, marks, distinct, evaluations) = count_walk(level);
        assert_eq!(changes, 175);
        assert_eq!(marks, distinct, "level {level}: a subchunk marked twice in one change");
        assert!(marks <= limit, "level {level}: marks {marks} limit {limit}");
        // A full scan of the reach would be about 2 x (2 x level + 3)^3 evaluations per change.
        let per_change = evaluations / changes as u64;
        assert!(per_change <= 2000, "level {level}: {per_change} evaluations per change");
    }
}
