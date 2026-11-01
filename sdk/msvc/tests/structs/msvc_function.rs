use std::ffi::c_void;

use amml_msvc::MsvcFunction;
use amml_msvc_fixture::{fx_function_call3, fx_sizeof_function};

#[repr(C)]
struct Opaque {
    value: i32,
}

#[test]
fn cpp_calls_the_closure_through_a_const_reference_with_each_argument() {
    // SAFETY: the probe returns a sizeof value.
    assert_eq!(unsafe { fx_sizeof_function() }, 64);
    let a = Opaque { value: 3 };
    let b = Opaque { value: 4 };
    let mut seen = Vec::new();
    let function = MsvcFunction::new(|first: *const c_void, second: *const c_void, flag: bool| {
        // SAFETY: the fixture passes references to the two live Opaque values.
        let (first, second) = unsafe { ((*first.cast::<Opaque>()).value, (*second.cast::<Opaque>()).value) };
        seen.push((first, second, flag));
        first != second
    });
    let pa = (&raw const a).cast::<c_void>();
    let pb = (&raw const b).cast::<c_void>();
    // SAFETY: the object is a live std::function of the probe's signature.
    let trues = unsafe { fx_function_call3(function.as_ptr(), pa, pb) };
    drop(function);

    assert_eq!(trues, 2);
    assert_eq!(seen, vec![(3, 4, true), (4, 3, false), (3, 3, true)]);
}

#[test]
fn the_cpp_object_stays_valid_when_the_rust_value_moves() {
    let a = Opaque { value: 1 };
    let function = MsvcFunction::new(|_: *const c_void, _: *const c_void, flag: bool| flag);
    let moved = [function];
    let pa = (&raw const a).cast::<c_void>();
    // SAFETY: the object is a live std::function of the probe's signature.
    let trues = unsafe { fx_function_call3(moved[0].as_ptr(), pa, pa) };
    assert_eq!(trues, 2);
}
