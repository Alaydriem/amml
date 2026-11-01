use amml_msvc::Optional;
use amml_msvc_fixture::{
    fx_optional_int_delete, fx_optional_int_make, fx_optional_int_read, fx_optional_vec3_delete, fx_optional_vec3_make,
    fx_optional_vec3_read, fx_sizeof_optional_int, fx_sizeof_optional_vec3,
};
use std::mem::size_of;

#[test]
fn optional_layout_and_values_match_for_small_and_large_payloads() {
    // SAFETY: the probes only return sizeof values.
    assert_eq!(size_of::<Optional<i32>>(), unsafe { fx_sizeof_optional_int() });
    // SAFETY: the probes only return sizeof values.
    assert_eq!(size_of::<Optional<[f64; 3]>>(), unsafe { fx_sizeof_optional_vec3() });
    // SAFETY: the pointer is to a live Optional<i32>, which mirrors std::optional<int>.
    assert_eq!(unsafe { fx_optional_int_read(&Optional::some(41) as *const _ as _) }, 41);
    // SAFETY: as above.
    assert_eq!(unsafe { fx_optional_int_read(&Optional::<i32>::none() as *const _ as _) }, -1);
    // SAFETY: the pointer is to a live Optional<[f64; 3]>.
    let sum = unsafe { fx_optional_vec3_read(&Optional::some([1.0, 2.0, 3.0]) as *const _ as _) };
    assert_eq!(sum, 6.0);
    // SAFETY: as above.
    assert_eq!(unsafe { fx_optional_vec3_read(&Optional::<[f64; 3]>::none() as *const _ as _) }, -1.0);
}

#[test]
fn reads_cpp_optionals() {
    // SAFETY: the probe returns live std::optional<int> objects.
    let (engaged, empty) = unsafe { (fx_optional_int_make(true, 12), fx_optional_int_make(false, 0)) };
    // SAFETY: both pointers are live std::optional<int>.
    unsafe {
        assert_eq!((&*(engaged as *const Optional<i32>)).get(), Some(&12));
        assert_eq!((&*(empty as *const Optional<i32>)).get(), None);
        fx_optional_int_delete(engaged);
        fx_optional_int_delete(empty);
    }

    // SAFETY: the probe returns live std::optional<std::array<double,3>> objects.
    let (engaged, empty) = unsafe { (fx_optional_vec3_make(true), fx_optional_vec3_make(false)) };
    // SAFETY: both pointers are live optionals of three doubles.
    unsafe {
        assert_eq!((&*(engaged as *const Optional<[f64; 3]>)).get(), Some(&[1.0, 2.0, 3.0]));
        assert_eq!((&*(empty as *const Optional<[f64; 3]>)).get(), None);
        fx_optional_vec3_delete(engaged);
        fx_optional_vec3_delete(empty);
    }
}
