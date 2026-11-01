use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml_abi::AMML_API_VERSION;
use amml_sdk::{Api, ModRuntime, amml_detour};

use crate::support::{FakeLoader, LINES};

extern "C" fn real_function(x: i32) -> i32 {
    x + 1000
}

amml_detour!(pub mod doubler(x: i32) -> i32 |original| {
    if x == 7 {
        panic!("seven is unlucky");
    }
    original(x) * 2
});

#[test]
fn a_detour_that_panics_once_logs_disables_and_the_original_still_runs() {
    let table = FakeLoader::current();
    // SAFETY: the table is live and full size.
    let api = unsafe { Api::from_raw(&raw const table, AMML_API_VERSION) }.unwrap();
    ModRuntime::install(api, "guard-test");
    doubler::GUARD.original().store(real_function as *mut c_void, Ordering::Release);

    // The body runs and wraps the original.
    assert_eq!(doubler::detour(1), 2002);
    assert!(LINES.lock().unwrap().is_empty());

    // The body panics: one log line, the original answers for this call.
    assert_eq!(doubler::detour(7), 1007);
    {
        let lines = LINES.lock().unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("guard-test 4 detour doubler panicked: seven is unlucky"));
    }
    assert!(ModRuntime::is_disabled());

    // Disabled: the body is skipped and no further line is written, even for the panicking input.
    assert_eq!(doubler::detour(1), 1001);
    assert_eq!(doubler::detour(7), 1007);
    assert_eq!(LINES.lock().unwrap().len(), 1);
}

static REAL_CALLS: AtomicUsize = AtomicUsize::new(0);

extern "C" fn counted_function(x: i32) -> i32 {
    REAL_CALLS.fetch_add(1, Ordering::SeqCst);
    x + 1
}

amml_detour!(pub mod late_panic(x: i32) -> i32 |original| {
    let result = original(x);
    panic!("after the original returned {result}");
});

#[test]
fn a_body_that_called_the_original_before_panicking_does_not_run_it_again() {
    let table = FakeLoader::current();
    // SAFETY: the table is live and full size.
    let api = unsafe { Api::from_raw(&raw const table, AMML_API_VERSION) }.unwrap();
    ModRuntime::install(api, "guard-test");
    late_panic::GUARD.original().store(counted_function as *mut c_void, Ordering::Release);

    assert_eq!(late_panic::detour(41), 42);
    assert_eq!(REAL_CALLS.load(Ordering::SeqCst), 1);
    assert!(ModRuntime::is_disabled());
}
