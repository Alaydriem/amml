use amml_runtime::HandlerCapture;

const HANDLER: usize = 0x1000;
const OTHER_HANDLER: usize = 0x2000;
const ID: usize = 0x3000;

#[test]
fn capture_is_cleared_at_world_leave() {
    let capture = HandlerCapture::new();
    assert!(capture.captured().is_none());
    capture.capture(HANDLER, ID);
    let captured = capture.captured().unwrap();
    assert_eq!((captured.handler, captured.id), (HANDLER, ID));
    assert_eq!(captured.thread, std::thread::current().id());

    capture.world_left();

    assert!(capture.captured().is_none());
    // A late call of the left world's handler is not kept; the next join captures again.
    assert!(!capture.capture(HANDLER, ID));
    assert!(capture.captured().is_none());
    capture.join_started();
    assert!(capture.capture(HANDLER, ID));
}

#[test]
fn generation_changes_with_the_handler() {
    let capture = HandlerCapture::new();
    assert!(capture.capture(HANDLER, ID));
    let first = capture.captured().unwrap().generation;
    assert_ne!(first, 0);

    // The same handler again, with another identifier: the same generation, the new identifier.
    assert!(!capture.capture(HANDLER, ID + 8));
    let again = capture.captured().unwrap();
    assert_eq!((again.generation, again.id), (first, ID + 8));

    assert!(capture.capture(OTHER_HANDLER, ID));
    let second = capture.captured().unwrap().generation;
    assert_ne!(second, first);

    // After a world leave and a new join the same handler is a new capture.
    capture.world_left();
    capture.join_started();
    assert!(capture.capture(OTHER_HANDLER, ID));
    let third = capture.captured().unwrap().generation;
    assert!(third != second && third != first);
}

#[test]
fn a_null_handler_is_not_captured() {
    let capture = HandlerCapture::new();
    assert!(!capture.capture(0, ID));
    assert!(capture.captured().is_none());
}

#[test]
fn join_started_forgets_the_held_handler() {
    let capture = HandlerCapture::new();
    assert!(capture.capture(HANDLER, ID));
    let first = capture.captured().unwrap().generation;

    // A join with no world leave before it (a missed exit hook): the old world's handler is not kept.
    capture.join_started();

    assert!(capture.captured().is_none());
    assert!(capture.capture(HANDLER, ID));
    assert_ne!(capture.captured().unwrap().generation, first);
}
