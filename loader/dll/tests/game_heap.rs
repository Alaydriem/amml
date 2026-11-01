use amml::{GameHeap, LoaderLog};

use crate::support::{HeapProbe, LogFile};

#[test]
fn new_and_delete_resolve_as_a_pair() {
    let both = GameHeap::new(Some(HeapProbe::alloc), Some(HeapProbe::free));
    assert!(both.is_available());
    let block = both.alloc(32);
    assert!(!block.is_null());
    // SAFETY: the block came from `alloc` with this size and is freed once.
    unsafe { both.free(block, 32) };
    assert_eq!(HeapProbe::counts(), (1, 1));

    let only_new = GameHeap::new(Some(HeapProbe::alloc), None);
    assert!(!only_new.is_available());
    assert!(only_new.alloc(32).is_null());
    // SAFETY: a null block is never read or freed when the heap is unavailable.
    unsafe { only_new.free(std::ptr::null_mut(), 32) };

    let only_delete = GameHeap::new(None, Some(HeapProbe::free));
    assert!(only_delete.alloc(32).is_null());
    // The unavailable heaps called neither function.
    assert_eq!(HeapProbe::counts(), (1, 1));
}

#[test]
fn a_deferred_check_runs_once_on_the_first_alloc_and_logs_game_heap_ok() {
    let logs = tempfile::tempdir().unwrap();
    LoaderLog::install(logs.path()).unwrap();
    let heap = GameHeap::new(Some(HeapProbe::alloc), Some(HeapProbe::free));

    heap.defer_check();
    assert_eq!(HeapProbe::counts(), (0, 0));

    let first = heap.alloc(16);
    assert_eq!(HeapProbe::counts(), (2, 1));
    let second = heap.alloc(16);
    assert_eq!(HeapProbe::counts(), (3, 1));
    // SAFETY: both blocks came from `alloc(16)` of this heap and are freed once.
    unsafe {
        heap.free(first, 16);
        heap.free(second, 16);
    }

    let log = LogFile::wait_for(&logs.path().join("loader.log"), "game heap ok");
    assert_eq!(log.matches("game heap ok").count(), 1, "{log}");
}

#[test]
fn a_check_at_load_is_not_repeated_by_alloc() {
    let heap = GameHeap::new(Some(HeapProbe::alloc), Some(HeapProbe::free));

    heap.check();
    assert_eq!(HeapProbe::counts(), (1, 1));
    heap.check();
    let block = heap.alloc(8);
    // SAFETY: the block came from `alloc(8)` of this heap and is freed once.
    unsafe { heap.free(block, 8) };

    assert_eq!(HeapProbe::counts(), (2, 2));
}
