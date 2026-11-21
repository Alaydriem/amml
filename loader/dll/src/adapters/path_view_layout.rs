/// Mirror of `Core::PathView`: a character pointer and a length.
#[repr(C)]
pub struct PathViewLayout {
    pub data: *const u8,
    pub size: usize,
}

const _: () = assert!(size_of::<PathViewLayout>() == 16);
