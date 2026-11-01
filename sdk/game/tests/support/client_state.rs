use std::ffi::c_void;

/// What a fake game object answers.
pub struct ClientState {
    pub player: *mut c_void,
    pub options: *mut c_void,
    pub world_active: bool,
}
