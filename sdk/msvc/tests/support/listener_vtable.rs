use amml_msvc::ScalarDeletingDestructor;
use std::ffi::c_void;

/// Slot order of the fixture's `Listener`: destructor, `on_value`, `on_name`.
#[repr(C)]
pub struct ListenerVtable {
    pub destructor: ScalarDeletingDestructor,
    pub on_value: unsafe extern "C" fn(this: *mut c_void, value: i32) -> i32,
    pub on_name: unsafe extern "C" fn(this: *mut c_void, name: *const amml_msvc::MsvcString),
}
