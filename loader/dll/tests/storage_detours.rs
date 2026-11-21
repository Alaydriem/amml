use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use amml::StorageDetours;

const FIRST: usize = 0x1000;
const SECOND: usize = 0x2000;

#[test]
fn a_shutdown_waits_for_a_reader_that_holds_a_storage_that_is_no_longer_live() {
    StorageDetours::set_live(FIRST);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let reader = std::thread::spawn(move || {
        StorageDetours::with_live(|storage| {
            entered_tx.send(storage).unwrap();
            release_rx.recv().unwrap();
            storage
        })
    });
    assert_eq!(entered_rx.recv().unwrap(), FIRST);
    // A second storage becomes the live one while the reader still holds the first.
    StorageDetours::set_live(SECOND);

    let done = Arc::new(AtomicBool::new(false));
    let seen = done.clone();
    let shutdown = std::thread::spawn(move || {
        let was_live = StorageDetours::withdraw(FIRST);
        seen.store(true, Ordering::SeqCst);
        was_live
    });
    std::thread::sleep(Duration::from_millis(200));
    assert!(!done.load(Ordering::SeqCst), "the shutdown returned while a reader was inside");

    release_tx.send(()).unwrap();
    assert_eq!(reader.join().unwrap(), Some(FIRST));
    assert!(!shutdown.join().unwrap());
    assert!(done.load(Ordering::SeqCst));
    // The second storage is still live, and its own shutdown withdraws it.
    assert_eq!(StorageDetours::with_live(|storage| storage), Some(SECOND));
    assert!(StorageDetours::withdraw(SECOND));
    assert_eq!(StorageDetours::with_live(|storage| storage), None);
}
