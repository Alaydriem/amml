use std::ffi::c_void;

use amml_msvc::FunctionRef;
use amml_msvc_fixture::fx_function_ref_call;

#[repr(C)]
struct Opaque {
    value: i32,
}

#[test]
fn cpp_calls_the_borrowed_closure_with_the_reference_and_both_pointers() {
    let mut a = Opaque { value: 7 };
    let mut b = Opaque { value: 9 };
    let pa = (&raw mut a).cast::<c_void>();
    let pb = (&raw mut b).cast::<c_void>();
    let mut calls = Vec::new();
    let mut closure = |target: *mut c_void, source: *mut c_void, owner: *mut c_void| {
        // SAFETY: the fixture passes a reference to a live Opaque as the first argument.
        let value = unsafe { (*target.cast::<Opaque>()).value };
        calls.push((value, source, owner));
        value == 7
    };
    let trues = {
        let reference = FunctionRef::new(&mut closure);
        // SAFETY: the reference has the shape of the fixture's function reference, and the closure outlives the call.
        unsafe { fx_function_ref_call(reference.as_ptr(), pa, pb) }
    };

    assert_eq!(trues, 1);
    assert_eq!(calls, vec![(7, pb, std::ptr::null_mut()), (9, std::ptr::null_mut(), pa)]);
}
