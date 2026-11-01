use crate::support::Fixture;
use amml_msvc_fixture::{fx_alloc_like_vector, fx_free_like_vector};

const SIZES: [usize; 5] = [1, 15, 4095, 4096, 1 << 20];

#[test]
fn rust_allocated_blocks_are_freed_by_cpp_at_every_size() {
    let heap = Fixture::heap();
    for bytes in SIZES {
        // SAFETY: the fixture heap is the real operator new/delete.
        let p = unsafe { heap.allocate(bytes) };
        // SAFETY: `p` points to `bytes` writable bytes.
        unsafe { std::ptr::write_bytes(p, 0xAB, bytes) };
        // SAFETY: `p` was allocated with the STL layout for `bytes`.
        unsafe { fx_free_like_vector(p, bytes) };
    }
}

#[test]
fn cpp_allocated_blocks_are_freed_by_rust_at_every_size() {
    let heap = Fixture::heap();
    for bytes in SIZES {
        // SAFETY: the probe allocates through std::allocator<char>.
        let p = unsafe { fx_alloc_like_vector(bytes) };
        // SAFETY: `p` came from std::allocator<char> for `bytes`.
        unsafe { heap.deallocate(p, bytes) };
    }
}

#[test]
fn big_blocks_are_32_byte_aligned() {
    let heap = Fixture::heap();
    // SAFETY: the fixture heap is the real operator new/delete.
    let p = unsafe { heap.allocate(4096) };
    assert_eq!(p as usize % 32, 0);
    // SAFETY: `p` was just allocated for 4096 bytes.
    unsafe { heap.deallocate(p, 4096) };
}

mod recording {
    use amml_msvc::MsvcHeap;
    use std::cell::RefCell;

    #[derive(Default)]
    struct Calls {
        new_sizes: Vec<usize>,
        new_ptrs: Vec<usize>,
        deleted: Vec<(usize, usize)>,
    }

    thread_local! {
        static CALLS: RefCell<Calls> = RefCell::new(Calls::default());
    }

    unsafe extern "C" fn recording_new(bytes: usize) -> *mut u8 {
        // SAFETY: the fixture probe returns the real operator new.
        let p = unsafe { (amml_msvc_fixture::fx_operator_new())(bytes) };
        CALLS.with(|c| {
            let mut c = c.borrow_mut();
            c.new_sizes.push(bytes);
            c.new_ptrs.push(p as usize);
        });
        p
    }

    unsafe extern "C" fn recording_delete(ptr: *mut u8, bytes: usize) {
        CALLS.with(|c| c.borrow_mut().deleted.push((ptr as usize, bytes)));
        // SAFETY: the fixture probe returns the real operator delete and `ptr` came from recording_new.
        unsafe { (amml_msvc_fixture::fx_operator_delete())(ptr, bytes) };
    }

    fn take() -> Calls {
        CALLS.with(|c| std::mem::take(&mut *c.borrow_mut()))
    }

    #[test]
    fn big_blocks_request_39_extra_bytes_store_the_raw_pointer_and_free_it_with_that_size() {
        let heap = MsvcHeap::new(recording_new, recording_delete);
        for bytes in [4096usize, 1 << 20] {
            take();
            // SAFETY: the recording pair forwards to the real operator new/delete.
            let p = unsafe { heap.allocate(bytes) };
            let made = take();
            assert_eq!(made.new_sizes, vec![bytes + 39]);
            // SAFETY: `p` is a big block, so the slot before it is readable.
            let stored = unsafe { p.cast::<usize>().sub(1).read() };
            assert_eq!(stored, made.new_ptrs[0]);
            // SAFETY: `p` was allocated for `bytes` by this heap.
            unsafe { heap.deallocate(p, bytes) };
            assert_eq!(take().deleted, vec![(made.new_ptrs[0], bytes + 39)]);
        }
    }

    #[test]
    fn small_blocks_pass_pointer_and_size_through() {
        let heap = MsvcHeap::new(recording_new, recording_delete);
        for bytes in [1usize, 4095] {
            take();
            // SAFETY: the recording pair forwards to the real operator new/delete.
            let p = unsafe { heap.allocate(bytes) };
            let made = take();
            assert_eq!(made.new_sizes, vec![bytes]);
            assert_eq!(made.new_ptrs[0], p as usize);
            // SAFETY: `p` was allocated for `bytes` by this heap.
            unsafe { heap.deallocate(p, bytes) };
            assert_eq!(take().deleted, vec![(p as usize, bytes)]);
        }
    }

    #[test]
    fn aligned_pointer_is_aligned_after_raw_and_inside_the_block() {
        for raw in 0x1000usize..0x1020 {
            let aligned = MsvcHeap::aligned_from_raw(raw);
            assert_eq!(aligned % 32, 0, "raw {raw:#x}");
            assert!(aligned >= raw + 8, "raw {raw:#x}");
            assert!(aligned <= raw + 39, "raw {raw:#x}");
        }
    }
}
