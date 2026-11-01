use std::ffi::c_void;

/// The vtable of a fake `Dimension`: slot 0 is reserved, slot 1 is `getDimensionId`. `DimensionType` is a
/// struct, so the function takes the hidden return pointer after `this` and gives it back, as MSVC x64 does.
#[repr(C)]
pub struct DimensionVtable {
    pub reserved: usize,
    pub get_dimension_id: extern "C" fn(*mut c_void, *mut i32) -> *mut i32,
}
