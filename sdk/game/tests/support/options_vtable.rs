use std::ffi::c_void;

/// Slots of a fake `IOptionRegistry`: 0 reserved, 1 `getHideHud`.
#[repr(C)]
pub struct OptionsVtable {
    pub reserved: usize,
    pub get_hide_hud: extern "C" fn(*mut c_void) -> u8,
}
