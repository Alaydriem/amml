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

mod events {
    use std::ffi::c_void;
    use std::sync::Mutex;

    use amml::{JoinEvents, ModListener, SessionAdapter};
    use amml_abi::{
        AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlStatus, AmmlStr, WorldStorageClosed, WorldStorageOpened,
    };

    use crate::support::{Harness, InfoGame, JoinGame, SessionGame};

    // The events a test listens to; a listener's `user` value is its index here.
    const EVENTS: [&str; 3] = ["frame.update", "world.storage_opened", "world.storage_closed"];

    static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

    fn text(view: AmmlStr) -> String {
        // SAFETY: the loader lends `len` bytes at `ptr` for the call.
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(view.ptr, view.len) }).into_owned()
    }

    unsafe extern "C" fn record(
        event: *mut AmmlEventHeader,
        context: *const AmmlEventContext,
        user: *mut c_void,
    ) -> AmmlStatus {
        // SAFETY: the loader passes a live header and context for the call.
        let (header, context) = unsafe { (&*event, &*context) };
        let detail = match EVENTS[user as usize] {
            // SAFETY: a `world.storage_opened` event is a `WorldStorageOpened`.
            "world.storage_opened" => format!(" path={}", text(unsafe { &*event.cast::<WorldStorageOpened>() }.path)),
            "world.storage_closed" => {
                // SAFETY: a `world.storage_closed` event is a `WorldStorageClosed`, which is its header.
                let size = unsafe { &*event.cast::<WorldStorageClosed>() }.header.size;
                format!(" size={size}")
            }
            _ => String::new(),
        };
        SEEN.lock()
            .unwrap()
            .push(format!("{} phase={} thread={}{detail}", EVENTS[user as usize], header.phase.0, context.thread.0));
        AmmlStatus::OK
    }

    // Installs the session and join adapters and listens to the frame and both storage events; the first frame opens
    // the gate and makes the test thread the game thread.
    fn start() -> Harness {
        let harness = Harness::start_with_entries(&JoinGame::entries(InfoGame::SIZE_2663));
        SessionAdapter::install(harness.state);
        JoinEvents::install(harness.state);
        for (index, event) in EVENTS.iter().enumerate() {
            let listener = ModListener { owner: 9, callback: record, user: index };
            harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap();
        }
        SessionGame::frame();
        SEEN.lock().unwrap().clear();
        harness
    }

    fn seen() -> Vec<String> {
        SEEN.lock().unwrap().clone()
    }

    #[test]
    fn storage_opened_carries_the_derived_path() {
        let harness = start();
        assert!(harness.state.events().is_available("world.storage_opened", 1));
        assert!(harness.state.events().is_available("world.storage_closed", 1));
        let game = amml_abi::AmmlThread::GAME.0;

        InfoGame::construct_storage("C:/worlds/My World");
        assert_eq!(InfoGame::shutdown_storage(), 1);
        // The game asks again after it is done; the event comes once.
        assert_eq!(InfoGame::shutdown_storage(), 1);

        assert_eq!(
            seen(),
            [
                format!("world.storage_opened phase=1 thread={game} path=C:/worlds/My World"),
                format!("world.storage_closed phase=1 thread={game} size=16"),
            ]
        );
    }

    #[test]
    fn storage_event_off_thread_is_raised_at_the_next_frame() {
        let _harness = start();
        let game = amml_abi::AmmlThread::GAME.0;

        std::thread::spawn(|| {
            InfoGame::construct_storage("C:/worlds/Other");
            InfoGame::shutdown_storage();
        })
        .join()
        .unwrap();
        assert!(seen().is_empty(), "an event was raised on the storage thread: {:?}", seen());

        SessionGame::frame();

        assert_eq!(
            seen(),
            [
                format!("frame.update phase=0 thread={game}"),
                format!("frame.update phase=1 thread={game}"),
                format!("world.storage_opened phase=1 thread={game} path=C:/worlds/Other"),
                format!("world.storage_closed phase=1 thread={game} size=16"),
            ]
        );
        // The frame after raises nothing more than its own two frame events.
        SEEN.lock().unwrap().clear();
        SessionGame::frame();
        assert_eq!(seen().len(), 2);
    }
}
