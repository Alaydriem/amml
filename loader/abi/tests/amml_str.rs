use amml_abi::{AmmlStatus, AmmlStr};

#[test]
fn bytes_that_are_not_utf8_are_rejected_at_both_ends() {
    let bad = [0x66u8, 0xff, 0xfe];
    assert_eq!(AmmlStr::from_bytes(&bad).unwrap_err(), AmmlStatus::INVALID_ARGUMENT);
    let view = AmmlStr { ptr: bad.as_ptr(), len: bad.len() };
    // SAFETY: `bad` outlives the view and `len` is its length.
    let read = unsafe { view.as_str() };
    assert_eq!(read.unwrap_err(), AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn text_survives_the_view() {
    let view = AmmlStr::from_str("core/héllo");
    // SAFETY: the view borrows a string literal.
    assert_eq!(unsafe { view.as_str() }.unwrap(), "core/héllo");
}

#[test]
fn null_pointer_is_empty_only_with_zero_length() {
    // SAFETY: a zero-length view reads no memory.
    assert_eq!(unsafe { AmmlStr::EMPTY.as_str() }.unwrap(), "");
    let bad = AmmlStr { ptr: core::ptr::null(), len: 4 };
    // SAFETY: the null pointer is rejected before any read.
    assert_eq!(unsafe { bad.as_str() }.unwrap_err(), AmmlStatus::INVALID_ARGUMENT);
}
