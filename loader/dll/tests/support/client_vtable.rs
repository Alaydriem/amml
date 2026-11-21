use std::ffi::c_void;

/// The `IClientInstance` vtable of the stand-in clients: slot 0 is `getLocalPlayer`, slot 1 `onStartJoinGame`.
#[repr(C)]
pub struct ClientVtable {
    pub local_player: extern "C" fn(*mut c_void) -> *mut c_void,
    pub start_join: extern "C" fn(*mut c_void, u64, u64, u64, u64, u64, u64, u64),
}
