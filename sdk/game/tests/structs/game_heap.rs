#![allow(unsafe_code)]

use std::alloc::{Layout, alloc, dealloc};
use std::ffi::c_void;
use std::sync::Mutex;

use amml_game::GameHeap;
use amml_sdk::amml_abi::{AMML_API_VERSION, AmmlApi, AmmlLogLevel, AmmlStatus, AmmlStr};
use amml_sdk::{Api, ModRuntime};

/// Every `(size, freed)` call the fake game heap saw: `false` for a new, `true` for a delete.
static CALLS: Mutex<Vec<(usize, bool)>> = Mutex::new(Vec::new());

/// A loader table whose heap functions allocate from the Rust heap and record their calls.
struct HeapLoader;

impl HeapLoader {
    unsafe extern "C" fn resolve(_: AmmlStr, _: *mut u64) -> AmmlStatus {
        AmmlStatus::MISSING_ENTRY
    }

    unsafe extern "C" fn hook(
        _: *const c_void,
        _: *const c_void,
        _: i32,
        _: *mut *const c_void,
        _: *mut u64,
    ) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn unhook(_: u64) -> AmmlStatus {
        AmmlStatus::OK
    }

    unsafe extern "C" fn log(_: AmmlStr, _: AmmlLogLevel, _: AmmlStr) {}

    unsafe extern "C" fn heap_new(size: usize) -> *mut u8 {
        CALLS.lock().unwrap().push((size, false));
        // SAFETY: the layout has a non-zero size; the callers here never ask for zero bytes.
        unsafe { alloc(Layout::from_size_align(size, 16).unwrap()) }
    }

    unsafe extern "C" fn heap_delete(ptr: *mut u8, size: usize) {
        CALLS.lock().unwrap().push((size, true));
        // SAFETY: `ptr` came from `heap_new` with this size and is freed once.
        unsafe { dealloc(ptr, Layout::from_size_align(size, 16).unwrap()) };
    }

    unsafe extern "C" fn mod_dir(_: AmmlStr, _: u8, _: *mut AmmlStr) -> AmmlStatus {
        AmmlStatus::FAILED
    }

    fn api() -> Api {
        let table = AmmlApi {
            size: AmmlApi::SIZE_V1,
            version: AMML_API_VERSION,
            module_base: 0x1000,
            resolve: Self::resolve,
            hook: Self::hook,
            unhook: Self::unhook,
            log: Self::log,
            heap_new: Self::heap_new,
            heap_delete: Self::heap_delete,
            mod_dir: Self::mod_dir,
        };
        // SAFETY: the table is live and full size.
        unsafe { Api::from_raw(&raw const table, AMML_API_VERSION) }.unwrap()
    }
}

// Each test runs in its own process under nextest, so the runtime install of one never reaches the other.
#[test]
fn there_is_no_heap_before_the_runtime_is_installed() {
    assert!(GameHeap::msvc().is_none());
}

#[test]
fn small_and_big_blocks_go_through_the_loader_heap_the_way_the_stl_allocates_them() {
    ModRuntime::install(HeapLoader::api(), "heap-test");
    let heap = GameHeap::msvc().unwrap();
    CALLS.lock().unwrap().clear();

    // SAFETY: the heap's functions are the loader's heap entries, backed by the Rust heap here.
    let small = unsafe { heap.allocate(24) };
    assert!(!small.is_null());
    // SAFETY: `small` came from `allocate(24)` and is freed once.
    unsafe { heap.deallocate(small, 24) };

    // SAFETY: as above.
    let big = unsafe { heap.allocate(8192) };
    assert!(!big.is_null());
    assert_eq!(big as usize % 32, 0);
    // SAFETY: `big` came from `allocate(8192)` and is freed once.
    unsafe { heap.deallocate(big, 8192) };

    // A big block carries the STL's extra bytes, in the allocation and in the matching delete.
    let extra = 8192 + size_of::<usize>() + 31;
    assert_eq!(*CALLS.lock().unwrap(), vec![(24, false), (24, true), (extra, false), (extra, true)]);
}
