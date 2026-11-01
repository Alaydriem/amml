use std::alloc::Layout;
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static FREES: AtomicUsize = AtomicUsize::new(0);

/// Fake game `operator new` / `operator delete` over the Rust allocator, with counters.
pub struct HeapProbe;

impl HeapProbe {
    pub unsafe extern "C-unwind" fn alloc(size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::SeqCst);
        // SAFETY: the layout has a non-zero size and alignment 1.
        unsafe { std::alloc::alloc(Layout::from_size_align(size.max(1), 1).unwrap()) }
    }

    pub unsafe extern "C" fn free(ptr: *mut u8, size: usize) {
        FREES.fetch_add(1, Ordering::SeqCst);
        // SAFETY: the pointer came from `new` with the same size.
        unsafe { std::alloc::dealloc(ptr, Layout::from_size_align(size.max(1), 1).unwrap()) }
    }

    pub fn counts() -> (usize, usize) {
        (ALLOCS.load(Ordering::SeqCst), FREES.load(Ordering::SeqCst))
    }
}
