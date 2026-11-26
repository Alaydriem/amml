use std::ffi::c_void;

use amml_abi::{AmmlOptionKey, AmmlOptionsTable, AmmlPersistence, AmmlStatus, AmmlStr};

unsafe extern "C" fn view_distance_chunks(_: *const c_void, _: *mut u32) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn get_int(_: AmmlOptionKey, _: *mut i32) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn set_int(_: AmmlStr, _: AmmlOptionKey, _: i32, _: AmmlPersistence) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn release_transient(_: AmmlStr, _: AmmlOptionKey) -> AmmlStatus {
    AmmlStatus::OK
}

fn table() -> AmmlOptionsTable {
    AmmlOptionsTable {
        size: AmmlOptionsTable::SIZE_V1,
        version: 1,
        view_distance_chunks,
        get_int,
        set_int,
        release_transient,
    }
}

#[test]
fn options_table_read_respects_size() {
    let table = table();
    assert_eq!(AmmlOptionsTable::SIZE_V1, 40);
    assert_eq!(core::mem::size_of::<AmmlOptionsTable>(), 40);

    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlOptionsTable::read(&raw const table) }.unwrap();
    assert_eq!(read.size, AmmlOptionsTable::SIZE_V1);

    // A table shorter than v1 does not cover every function: the reader refuses it.
    let mut short = table;
    short.size = AmmlOptionsTable::SIZE_V1 - 8;
    // SAFETY: the table lives for the call and the reader reads no byte past the size it finds.
    assert!(unsafe { AmmlOptionsTable::read(&raw const short) }.is_none());
    // SAFETY: a null table is refused before it is read.
    assert!(unsafe { AmmlOptionsTable::read(core::ptr::null()) }.is_none());

    // A table that claims to be longer than this reader's struct is read as far as the struct goes.
    let mut grown = table;
    grown.size = AmmlOptionsTable::SIZE_V1 + 64;
    // SAFETY: the claimed bytes past the struct are never read.
    let read = unsafe { AmmlOptionsTable::read(&raw const grown) }.unwrap();
    assert_eq!(read.size, AmmlOptionsTable::SIZE_V1 + 64);
}
