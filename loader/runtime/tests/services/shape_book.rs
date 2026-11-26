use amml_runtime::{ShapeBook, ShapeKind, ShapeRefusal, ShapeRequest, ShapeSend};

const MOD_A: usize = 1;
const MOD_B: usize = 2;
const GENERATION: u32 = 1;

fn draw(id: u64, kind: ShapeKind) -> ShapeRequest {
    ShapeRequest { id, kind: Some(kind) }
}

fn remove(id: u64) -> ShapeRequest {
    ShapeRequest { id, kind: None }
}

// The game network id the book gave for the one draw of `sends`.
fn drawn(sends: &[ShapeSend]) -> u64 {
    assert_eq!(sends.len(), 1, "{sends:?}");
    assert!(sends[0].kind.is_some(), "{sends:?}");
    sends[0].network_id
}

#[test]
fn ids_of_two_mods_do_not_collide() {
    let mut book = ShapeBook::new();

    let a = drawn(&book.apply(GENERATION, MOD_A, &[draw(1, ShapeKind::Line)]).unwrap());
    let b = drawn(&book.apply(GENERATION, MOD_B, &[draw(1, ShapeKind::Line)]).unwrap());

    assert_ne!(a, b);
    assert!(a >= ShapeBook::ID_BASE && b >= ShapeBook::ID_BASE);
    // A remove of mod B's id 1 removes mod B's shape only.
    let removed = book.apply(GENERATION, MOD_B, &[remove(1)]).unwrap();
    assert_eq!(removed, [ShapeSend { op: 0, network_id: b, kind: None }]);
    assert_eq!((book.live(MOD_A), book.live(MOD_B)), (1, 0));
    // A draw again of mod A's id 1 moves the same game shape.
    let moved = book.apply(GENERATION, MOD_A, &[draw(1, ShapeKind::Line)]).unwrap();
    assert_eq!(moved, [ShapeSend { op: 0, network_id: a, kind: Some(ShapeKind::Line) }]);
}

#[test]
fn disable_removes_the_live_shapes_of_that_mod_only() {
    let mut book = ShapeBook::new();
    let sends = book.apply(GENERATION, MOD_A, &[draw(1, ShapeKind::Line), draw(2, ShapeKind::Box), remove(2)]).unwrap();
    let a1 = sends[0].network_id;
    book.apply(GENERATION, MOD_B, &[draw(1, ShapeKind::Arrow)]).unwrap();

    // Only mod A's live shape goes: its id 2 was removed in the batch already.
    assert_eq!(book.disable(MOD_A, GENERATION), [a1]);
    assert_eq!((book.live(MOD_A), book.live(MOD_B)), (0, 1));
    assert!(book.disable(MOD_A, GENERATION).is_empty());

    // A disable after the handler changed sends nothing: the game dropped the shapes with the old handler.
    assert!(book.disable(MOD_B, GENERATION + 1).is_empty());
    assert_eq!(book.live(MOD_B), 0);
}

#[test]
fn world_leave_forgets_every_shape() {
    let mut book = ShapeBook::new();
    let first = drawn(&book.apply(GENERATION, MOD_A, &[draw(1, ShapeKind::Box)]).unwrap());
    book.apply(GENERATION, MOD_B, &[draw(7, ShapeKind::Line)]).unwrap();

    book.world_left();

    assert_eq!((book.live(MOD_A), book.live(MOD_B)), (0, 0));
    assert!(book.disable(MOD_A, GENERATION).is_empty());
    // A remove of a forgotten id sends nothing; a draw gets a new game id, never one used before.
    assert!(book.apply(GENERATION, MOD_A, &[remove(1)]).unwrap().is_empty());
    let again = drawn(&book.apply(GENERATION, MOD_A, &[draw(1, ShapeKind::Box)]).unwrap());
    assert_ne!(again, first);
}

#[test]
fn a_batch_over_the_maximum_is_refused_and_changes_nothing() {
    let mut book = ShapeBook::new();
    let requests: Vec<ShapeRequest> = (0..=ShapeBook::MAX_BATCH as u64).map(|id| draw(id, ShapeKind::Line)).collect();

    let refused = book.apply(GENERATION, MOD_A, &requests);

    assert_eq!(refused, Err(ShapeRefusal::TooMany { count: ShapeBook::MAX_BATCH + 1, max: ShapeBook::MAX_BATCH }));
    assert_eq!(book.live(MOD_A), 0);
    assert_eq!(book.apply(GENERATION, MOD_A, &requests[..ShapeBook::MAX_BATCH]).unwrap().len(), ShapeBook::MAX_BATCH);
}

#[test]
fn a_new_handler_generation_forgets_the_shapes_of_the_old_one() {
    let mut book = ShapeBook::new();
    let old = drawn(&book.apply(GENERATION, MOD_A, &[draw(1, ShapeKind::Line)]).unwrap());

    // The remove of a shape the game dropped with the old handler sends nothing.
    assert!(book.apply(GENERATION + 1, MOD_A, &[remove(1)]).unwrap().is_empty());
    let new = drawn(&book.apply(GENERATION + 1, MOD_A, &[draw(1, ShapeKind::Line)]).unwrap());
    assert_ne!(new, old);
}

#[test]
fn a_kind_change_removes_the_old_shape_and_draws_a_new_one() {
    let mut book = ShapeBook::new();
    let line = drawn(&book.apply(GENERATION, MOD_A, &[draw(1, ShapeKind::Line)]).unwrap());

    let sends = book.apply(GENERATION, MOD_A, &[draw(1, ShapeKind::Arrow)]).unwrap();

    assert_eq!(sends.len(), 2, "{sends:?}");
    assert_eq!(sends[0], ShapeSend { op: 0, network_id: line, kind: None });
    assert_eq!((sends[1].op, sends[1].kind), (0, Some(ShapeKind::Arrow)));
    assert_ne!(sends[1].network_id, line);
    assert_eq!(book.live(MOD_A), 1);
}
