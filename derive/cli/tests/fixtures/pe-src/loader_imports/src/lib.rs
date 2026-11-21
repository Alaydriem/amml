use core::ffi::c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
}

#[unsafe(no_mangle)]
pub extern "C" fn amml_probe() -> *mut c_void {
    // SAFETY: a null name gives the handle of the process exe; the name is a NUL-terminated literal.
    unsafe { GetProcAddress(GetModuleHandleW(core::ptr::null()), c"amml_probe".as_ptr().cast()) }
}
