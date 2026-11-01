use core::ffi::c_void;
use core::mem::offset_of;

use amml_abi::{AMML_API_VERSION, AmmlApi, AmmlLogLevel, AmmlStatus, AmmlStr};

struct Stubs;

impl Stubs {
    unsafe extern "C" fn resolve(_: AmmlStr, _: *mut u64) -> AmmlStatus {
        AmmlStatus::OK
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

    unsafe extern "C" fn heap_new(_: usize) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe extern "C" fn heap_delete(_: *mut u8, _: usize) {}

    unsafe extern "C" fn mod_dir(_: AmmlStr, _: u8, _: *mut AmmlStr) -> AmmlStatus {
        AmmlStatus::OK
    }

    fn api(size: u32) -> AmmlApi {
        AmmlApi {
            size,
            version: AMML_API_VERSION,
            module_base: 0,
            resolve: Self::resolve,
            hook: Self::hook,
            unhook: Self::unhook,
            log: Self::log,
            heap_new: Self::heap_new,
            heap_delete: Self::heap_delete,
            mod_dir: Self::mod_dir,
        }
    }
}

#[test]
fn full_table_supports_every_field() {
    let api = Stubs::api(AmmlApi::SIZE_V1);
    assert!(api.supports(offset_of!(AmmlApi, resolve), size_of::<usize>()));
    assert!(api.supports(offset_of!(AmmlApi, mod_dir), size_of::<usize>()));
}

#[test]
fn field_beyond_size_is_unsupported() {
    let older = Stubs::api(offset_of!(AmmlApi, mod_dir) as u32);
    assert!(older.supports(offset_of!(AmmlApi, heap_delete), size_of::<usize>()));
    assert!(!older.supports(offset_of!(AmmlApi, mod_dir), size_of::<usize>()));
}

#[test]
fn field_straddling_size_is_unsupported() {
    let api = Stubs::api(offset_of!(AmmlApi, mod_dir) as u32 + 4);
    assert!(!api.supports(offset_of!(AmmlApi, mod_dir), size_of::<usize>()));
}

#[test]
fn overflowing_range_is_unsupported() {
    let api = Stubs::api(AmmlApi::SIZE_V1);
    assert!(!api.supports(usize::MAX, 2));
}

#[repr(C, align(8))]
struct Leading {
    size: u32,
    version: u32,
}

#[test]
fn header_reads_a_short_table_without_touching_the_rest() {
    let short = Leading { size: 8, version: 1 };
    let ptr = (&raw const short).cast::<AmmlApi>();
    // SAFETY: `short` holds the two leading fields and is 8-aligned.
    assert_eq!(unsafe { AmmlApi::header(ptr) }, Some((8, 1)));
}

#[test]
fn header_of_a_null_table_is_none() {
    // SAFETY: a null pointer is checked before any read.
    assert_eq!(unsafe { AmmlApi::header(core::ptr::null()) }, None);
}
