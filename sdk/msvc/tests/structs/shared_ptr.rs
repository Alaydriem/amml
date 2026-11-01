use crate::support::Fixture;
use amml_msvc::SharedPtr;
use amml_msvc_fixture::{
    fx_blocks_freed, fx_shared_hold_weak, fx_shared_read, fx_shared_use_count, fx_sizeof_shared, fx_tracked_destroyed,
    fx_weak_expired, fx_weak_release,
};
use std::ffi::c_void;
use std::mem::{MaybeUninit, size_of};

#[test]
fn shared_ptr_size_matches() {
    // SAFETY: the probe only returns a sizeof value.
    assert_eq!(size_of::<SharedPtr<i32>>(), unsafe { fx_sizeof_shared() });
}

#[test]
fn destructor_runs_once_when_the_last_reference_is_released() {
    let mut a = Fixture::make_shared(5);
    // SAFETY: `a` holds a live use.
    let mut b = unsafe { a.clone_ref() };
    assert_eq!(a.use_count(), 2);
    assert_eq!(b.get(), Some(&5));
    // SAFETY: `a` holds a live use.
    unsafe { a.release() };
    assert_eq!(a.get(), None);
    // SAFETY: the probe reads the live shared_ptr that `b` mirrors.
    assert_eq!(unsafe { fx_shared_use_count(&b as *const _ as *const c_void) }, 1);
    // SAFETY: the probe only reads a counter.
    assert_eq!(unsafe { fx_tracked_destroyed() }, 0);
    // SAFETY: `b` holds the last use.
    unsafe { b.release() };
    // SAFETY: the probe only reads a counter.
    assert_eq!(unsafe { fx_tracked_destroyed() }, 1);
    // SAFETY: the probe only reads a counter.
    assert_eq!(unsafe { fx_blocks_freed() }, 1);
}

#[test]
fn control_block_survives_while_a_weak_reference_exists() {
    let mut shared = Fixture::make_shared(9);
    let mut weak = MaybeUninit::<[*mut c_void; 2]>::uninit();
    // SAFETY: the probe copies from a live shared_ptr into 16 bytes of storage.
    unsafe { fx_shared_hold_weak(&shared as *const _ as *const c_void, weak.as_mut_ptr().cast()) };
    // SAFETY: `shared` holds the only use.
    unsafe { shared.release() };
    // SAFETY: the probes read counters and the live weak_ptr.
    unsafe {
        assert_eq!(fx_tracked_destroyed(), 1);
        assert!(fx_weak_expired(weak.as_ptr().cast()));
        assert_eq!(fx_blocks_freed(), 0);
    }
    // SAFETY: the probe destroys the weak_ptr it constructed.
    unsafe { fx_weak_release(weak.as_mut_ptr().cast()) };
    // SAFETY: the probes only read counters.
    unsafe {
        assert_eq!(fx_tracked_destroyed(), 1);
        assert_eq!(fx_blocks_freed(), 1);
    }
}

#[test]
fn cpp_reads_a_rust_copy_and_the_value_lives_until_the_last_release() {
    let mut a = Fixture::make_shared(77);
    // SAFETY: `a` holds a live use.
    let mut b = unsafe { a.clone_ref() };
    // SAFETY: the probe reads the live shared_ptr that `b` mirrors.
    assert_eq!(unsafe { fx_shared_read(&b as *const _ as *const c_void) }, 77);
    // SAFETY: `b` holds a live use.
    unsafe { b.release() };
    // SAFETY: the probe only reads a counter.
    assert_eq!(unsafe { fx_tracked_destroyed() }, 0);
    // SAFETY: `a` holds the last use.
    unsafe { a.release() };
    // SAFETY: the probe only reads a counter.
    assert_eq!(unsafe { fx_tracked_destroyed() }, 1);
}
