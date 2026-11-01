use std::ffi::c_void;

use amml_abi::{AmmlLogLevel, AmmlStatus, AmmlStr};

use crate::support::{Harness, LogFile};

#[test]
fn invalid_calls_return_a_status_and_an_unknown_log_level_is_info() {
    let harness = Harness::start();
    // SAFETY: the table lives for the process and has the full v1 size.
    let api = unsafe { &*harness.state.api() };

    // SAFETY: each call passes live views; the null pointers are the cases under test.
    unsafe {
        assert_eq!((api.resolve)(AmmlStr::from_str("Echo.found"), std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
        let bad = AmmlStr { ptr: std::ptr::null(), len: 4 };
        let mut out = 0u64;
        assert_eq!((api.resolve)(bad, &mut out), AmmlStatus::INVALID_ARGUMENT);
        let mut handle = 0u64;
        assert_eq!(
            (api.hook)(std::ptr::null(), std::ptr::null(), 0, std::ptr::null_mut(), &mut handle),
            AmmlStatus::INVALID_ARGUMENT
        );
        assert_eq!(
            (api.hook)(std::ptr::null::<c_void>(), std::ptr::null(), 0, std::ptr::null_mut(), std::ptr::null_mut()),
            AmmlStatus::INVALID_ARGUMENT
        );
        // Valid pointers, but no mod callback is running: hooks are refused before the engine sees them.
        let marker = 0u8;
        let slot: *const u8 = std::ptr::null();
        let at = (&raw const marker).cast::<c_void>();
        assert_eq!(
            (api.hook)(at, at.wrapping_add(1), 0, (&raw const slot).cast_mut().cast(), &mut handle),
            AmmlStatus::FAILED
        );
        assert_eq!((api.unhook)(999), AmmlStatus::FAILED);
        let mut dir = AmmlStr::EMPTY;
        assert_eq!((api.mod_dir)(AmmlStr::from_str("nobody"), 0, &mut dir), AmmlStatus::INVALID_ARGUMENT);
        assert_eq!((api.mod_dir)(AmmlStr::from_str("nobody"), 9, &mut dir), AmmlStatus::INVALID_ARGUMENT);
        (api.log)(AmmlStr::from_str("probe"), AmmlLogLevel(99), AmmlStr::from_str("level ninety-nine"));
        (api.log)(AmmlStr::from_str("../escape"), AmmlLogLevel::WARN, AmmlStr::from_str("odd module name"));
    }

    let probe = LogFile::wait_for(&harness.logs().join("probe.log"), "level ninety-nine");
    assert!(probe.contains("INFO  level ninety-nine"), "{probe}");
    let loader = LogFile::wait_for(&harness.logs().join("loader.log"), "odd module name");
    assert!(loader.contains("WARN  odd module name"), "{loader}");
    assert!(!harness.root.path().join("escape.log").exists());
}

#[test]
fn module_names_that_differ_only_by_case_share_one_log_file() {
    let harness = Harness::start();
    // SAFETY: the table lives for the process and has the full v1 size.
    let api = unsafe { &*harness.state.api() };

    // SAFETY: each call passes live views.
    unsafe {
        (api.log)(AmmlStr::from_str("Echo"), AmmlLogLevel::INFO, AmmlStr::from_str("upper case line"));
        (api.log)(AmmlStr::from_str("echo"), AmmlLogLevel::INFO, AmmlStr::from_str("lower case line"));
    }

    let text = LogFile::wait_for(&harness.logs().join("echo.log"), "lower case line");
    assert!(text.contains("upper case line") && text.contains("lower case line"), "{text}");
    let names: Vec<String> = std::fs::read_dir(harness.logs())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.eq_ignore_ascii_case("echo.log"))
        .collect();
    assert_eq!(names, ["echo.log"]);
}
