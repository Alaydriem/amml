#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(key: i32) -> i16;
}

#[unsafe(no_mangle)]
pub extern "C" fn amml_probe(key: i32) -> i16 {
    // SAFETY: GetAsyncKeyState takes any integer and reads no memory of ours.
    unsafe { GetAsyncKeyState(key) }
}
