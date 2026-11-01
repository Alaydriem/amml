use std::ffi::c_void;

/// Slots of a fake `ClientInstance`: 0 reserved, then the five calls in this order.
#[repr(C)]
pub struct ClientVtable {
    pub reserved: usize,
    pub get_local_player: extern "C" fn(*mut c_void) -> *mut c_void,
    pub is_world_active: extern "C" fn(*mut c_void) -> u8,
    pub is_showing_menu: extern "C" fn(*mut c_void) -> u8,
    pub get_mouse_grabbed: extern "C" fn(*mut c_void) -> u8,
    pub get_options: extern "C" fn(*mut c_void) -> *mut c_void,
}
