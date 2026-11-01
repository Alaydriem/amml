/// The shape of the client tick detour: `(this, init_finished, extra) -> u8`.
///
/// `extra` carries the 26.60 third argument (r8) bit for bit. 26.51 ignores r8, so one declaration
/// works on both builds.
pub type ClientUpdateFn = unsafe extern "C" fn(usize, u8, u64) -> u8;
