use amml_runtime::{ClipBook, ClipKind, ClipStep};

const FIRST: usize = 1;
const SECOND: usize = 2;

fn active_book() -> ClipBook {
    let mut book = ClipBook::new();
    book.set_active(true);
    book
}

#[test]
fn focus_loss_releases_and_keeps_the_request() {
    let mut book = active_book();
    assert_eq!(book.request(FIRST, Some(ClipKind::Centre)), ClipStep::Apply(ClipKind::Centre));

    assert_eq!(book.set_active(false), ClipStep::Release);
    assert_eq!(book.set_active(false), ClipStep::Keep, "nothing is held to release twice");
    assert_eq!(book.moved(), ClipStep::Keep, "a move of an inactive window sets no clip");
}

#[test]
fn focus_gain_applies_again() {
    let mut book = active_book();
    book.request(FIRST, Some(ClipKind::Centre));
    book.set_active(false);

    assert_eq!(book.set_active(true), ClipStep::Apply(ClipKind::Centre));
    assert_eq!(book.moved(), ClipStep::Apply(ClipKind::Centre), "a size or a move applies at the new centre");
}

#[test]
fn a_request_while_inactive_waits_for_the_focus() {
    let mut book = ClipBook::new();
    assert_eq!(book.request(FIRST, Some(ClipKind::Centre)), ClipStep::Keep);
    assert_eq!(book.set_active(true), ClipStep::Apply(ClipKind::Centre));
}

#[test]
fn none_drops_the_request() {
    let mut book = active_book();
    book.request(FIRST, Some(ClipKind::Centre));

    assert_eq!(book.request(FIRST, None), ClipStep::Release);
    assert_eq!(book.set_active(true), ClipStep::Keep);
}

#[test]
fn disable_drops_the_clip() {
    let mut book = active_book();
    book.request(FIRST, Some(ClipKind::Centre));

    assert_eq!(book.end_owner(FIRST), ClipStep::Release);
    assert_eq!(book.end_owner(FIRST), ClipStep::Keep, "a second end finds nothing");
    assert_eq!(book.set_active(true), ClipStep::Keep, "the request is gone, not only released");
}

#[test]
fn last_request_wins_and_the_other_comes_back() {
    let mut book = active_book();
    book.request(FIRST, Some(ClipKind::Centre));
    book.request(SECOND, Some(ClipKind::Centre));

    assert_eq!(book.end_owner(SECOND), ClipStep::Keep, "the first mod's request holds the clip that is set");
    assert_eq!(book.set_active(false), ClipStep::Release);
    assert_eq!(book.set_active(true), ClipStep::Apply(ClipKind::Centre), "the first mod's request survived");
    assert_eq!(book.end_owner(FIRST), ClipStep::Release);
}

#[test]
fn a_dropped_first_request_does_not_release_the_last() {
    let mut book = active_book();
    book.request(FIRST, Some(ClipKind::Centre));
    book.request(SECOND, Some(ClipKind::Centre));

    assert_eq!(book.end_owner(FIRST), ClipStep::Keep);
    assert_eq!(book.end_owner(SECOND), ClipStep::Release);
}

#[test]
fn a_destroyed_window_releases_and_keeps_the_requests() {
    let mut book = active_book();
    book.request(FIRST, Some(ClipKind::Centre));

    assert_eq!(book.window_gone(), ClipStep::Release);
    assert_eq!(book.set_active(true), ClipStep::Apply(ClipKind::Centre));
}
