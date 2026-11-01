use std::ffi::c_void;

use amml_game::RawObject;

#[test]
fn a_null_vtable_pointer_gives_none_and_no_slot_is_read() {
    // An object whose first qword (the vtable pointer) is null. Reading a slot of it would fault.
    let object: [usize; 2] = [0, 0];

    // SAFETY: the object is readable for a qword; the null vtable is checked before any slot read.
    let slot = unsafe { RawObject::slot((&raw const object).cast::<c_void>(), 3) };

    assert!(slot.is_none());
}

#[test]
fn a_null_function_gives_none_or_false_and_is_not_called() {
    let mut object = 0_u64;
    let this = (&raw mut object).cast::<std::ffi::c_void>();

    // SAFETY: a null function is refused before any call.
    assert!(unsafe { amml_game::RawObject::call_pointer(std::ptr::null(), this) }.is_none());
    // SAFETY: as above.
    assert!(!unsafe { amml_game::RawObject::call_bool(std::ptr::null(), this) });
}
