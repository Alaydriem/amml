use std::path::PathBuf;
use std::time::{Duration, Instant};

use amml_abi::AmmlShotLayers;
use amml_runtime::{CapturePoint, ShotBook};

const PRESENTS: u64 = 120;
const WAIT: Duration = Duration::from_secs(5);

// Asks for a shot at present 0, now.
fn ask(book: &ShotBook, owner: usize, path: &str, layers: AmmlShotLayers) -> u64 {
    book.request(owner, PathBuf::from(path), layers, 0, Instant::now())
}

#[test]
fn a_shot_is_due_once_at_its_point() {
    let book = ShotBook::new();
    let id = ask(&book, 1, "a.png", AmmlShotLayers::WITH_OVERLAYS);
    assert!(book.due(CapturePoint::BeforeListeners).is_empty());
    let due = book.due(CapturePoint::AfterNormalBand);
    assert_eq!(due.len(), 1);
    assert_eq!((due[0].id, due[0].owner, due[0].path.clone()), (id, 1, PathBuf::from("a.png")));
    assert!(book.due(CapturePoint::AfterNormalBand).is_empty());
}

#[test]
fn a_game_only_shot_is_due_before_the_listeners() {
    let book = ShotBook::new();
    ask(&book, 1, "g.png", AmmlShotLayers::GAME_ONLY);
    assert!(book.due(CapturePoint::AfterNormalBand).is_empty());
    assert_eq!(book.due(CapturePoint::BeforeListeners).len(), 1);
}

#[test]
fn ids_grow_and_requests_come_oldest_first() {
    let book = ShotBook::new();
    let first = ask(&book, 1, "1.png", AmmlShotLayers::GAME_ONLY);
    let second = ask(&book, 2, "2.png", AmmlShotLayers::GAME_ONLY);
    assert_ne!(first, second);
    let ids: Vec<u64> = book.due(CapturePoint::BeforeListeners).iter().map(|shot| shot.id).collect();
    assert_eq!(ids, vec![first, second]);
}

#[test]
fn dropping_an_owner_removes_only_its_requests() {
    let book = ShotBook::new();
    ask(&book, 1, "1.png", AmmlShotLayers::GAME_ONLY);
    let kept = ask(&book, 2, "2.png", AmmlShotLayers::GAME_ONLY);
    book.drop_owner(1);
    let due = book.due(CapturePoint::BeforeListeners);
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].id, kept);
}

#[test]
fn the_book_is_empty_once_every_request_was_due_or_dropped() {
    let book = ShotBook::new();
    assert!(book.is_empty());
    ask(&book, 1, "1.png", AmmlShotLayers::GAME_ONLY);
    ask(&book, 2, "2.png", AmmlShotLayers::WITH_OVERLAYS);
    assert!(!book.is_empty());
    book.due(CapturePoint::BeforeListeners);
    assert!(!book.is_empty());
    book.drop_owner(2);
    assert!(book.is_empty());
}

#[test]
fn a_shot_expires_after_its_presents_or_its_wait() {
    let book = ShotBook::new();
    let asked = Instant::now();
    let by_presents = book.request(1, PathBuf::from("p.png"), AmmlShotLayers::GAME_ONLY, 10, asked);
    let by_time = book.request(2, PathBuf::from("t.png"), AmmlShotLayers::WITH_OVERLAYS, 200, asked);

    assert!(book.expired(10 + PRESENTS - 1, asked, PRESENTS, WAIT).is_empty());
    let first: Vec<u64> = book.expired(10 + PRESENTS, asked, PRESENTS, WAIT).iter().map(|shot| shot.id).collect();
    assert_eq!(first, vec![by_presents]);
    assert!(book.expired(201, asked + WAIT - Duration::from_millis(1), PRESENTS, WAIT).is_empty());
    let second: Vec<u64> = book.expired(201, asked + WAIT, PRESENTS, WAIT).iter().map(|shot| shot.id).collect();
    assert_eq!(second, vec![by_time]);
    assert!(book.is_empty());
}

#[test]
fn take_all_hands_out_every_request_once() {
    let book = ShotBook::new();
    let first = ask(&book, 1, "1.png", AmmlShotLayers::GAME_ONLY);
    let second = ask(&book, 2, "2.png", AmmlShotLayers::WITH_OVERLAYS);

    let ids: Vec<u64> = book.take_all().iter().map(|shot| shot.id).collect();

    assert_eq!(ids, vec![first, second]);
    assert!(book.take_all().is_empty());
    assert!(book.due(CapturePoint::BeforeListeners).is_empty());
}
