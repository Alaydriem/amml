use amml_abi::{AmmlHoldRequest, AmmlStatus, AmmlStr, AmmlTimeHold, AmmlTimeState, AmmlTimeTable};

unsafe extern "C" fn hold(_: AmmlStr, _: *const AmmlHoldRequest, _: *mut AmmlTimeHold) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn step_us(_: AmmlTimeHold, _: i64) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn release(_: AmmlTimeHold) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn state(_: *mut AmmlTimeState) -> AmmlStatus {
    AmmlStatus::OK
}

pub(crate) fn table() -> AmmlTimeTable {
    AmmlTimeTable { size: AmmlTimeTable::SIZE_V1, version: 1, hold, step_us, release, state }
}

#[test]
fn time_table_read_respects_size() {
    let table = table();
    assert_eq!(core::mem::size_of::<AmmlTimeTable>(), AmmlTimeTable::SIZE_V1 as usize);

    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlTimeTable::read(&raw const table) }.unwrap();
    assert_eq!(read.size, AmmlTimeTable::SIZE_V1);

    // A table shorter than v1 does not cover every function: the reader refuses it.
    let mut short = table;
    short.size = AmmlTimeTable::SIZE_V1 - 8;
    // SAFETY: the table lives for the call and the reader reads no byte past the size it finds.
    assert!(unsafe { AmmlTimeTable::read(&raw const short) }.is_none());
    // SAFETY: a null table is refused before it is read.
    assert!(unsafe { AmmlTimeTable::read(core::ptr::null()) }.is_none());

    // A table that claims to be longer than this reader's struct is read as far as the struct goes.
    let mut grown = table;
    grown.size = AmmlTimeTable::SIZE_V1 + 64;
    // SAFETY: the claimed bytes past the struct are never read.
    let read = unsafe { AmmlTimeTable::read(&raw const grown) }.unwrap();
    assert_eq!(read.size, AmmlTimeTable::SIZE_V1 + 64);
}
