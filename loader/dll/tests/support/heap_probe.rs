use std::alloc::Layout;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static FREES: AtomicUsize = AtomicUsize::new(0);
// While true, `alloc` gives null, as a game heap that is out of memory would.
static FAILING: AtomicBool = AtomicBool::new(false);
// Each block given (address, size) and each block freed (address, size), in order.
static ALLOCATED: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());
static FREED: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

/// Fake game `operator new` / `operator delete` over the Rust allocator, with counters.
pub struct HeapProbe;

impl HeapProbe {
    pub unsafe extern "C-unwind" fn alloc(size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::SeqCst);
        if FAILING.load(Ordering::SeqCst) {
            return std::ptr::null_mut();
        }
        // SAFETY: the layout has a non-zero size and alignment 1.
        let block = unsafe { std::alloc::alloc(Layout::from_size_align(size.max(1), 1).unwrap()) };
        ALLOCATED.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).push((block as usize, size));
        block
    }

    pub unsafe extern "C" fn free(ptr: *mut u8, size: usize) {
        FREES.fetch_add(1, Ordering::SeqCst);
        FREED.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).push((ptr as usize, size));
        // SAFETY: the pointer came from `new` with the same size.
        unsafe { std::alloc::dealloc(ptr, Layout::from_size_align(size.max(1), 1).unwrap()) }
    }

    /// Makes every later `alloc` give null (`true`) or memory again (`false`).
    pub fn set_failing(failing: bool) {
        FAILING.store(failing, Ordering::SeqCst);
    }

    /// Every block `alloc` gave so far, as (address, size).
    pub fn allocated() -> Vec<(usize, usize)> {
        ALLOCATED.lock().unwrap().clone()
    }

    /// Every block `free` got so far, as (address, size).
    pub fn freed() -> Vec<(usize, usize)> {
        FREED.lock().unwrap().clone()
    }

    pub fn counts() -> (usize, usize) {
        (ALLOCS.load(Ordering::SeqCst), FREES.load(Ordering::SeqCst))
    }
}
