use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

pub struct PackageContext;

impl PackageContext {
    pub fn new() -> PackageContext {
        PackageContext
    }

    // True only when the process has a package identity: the call asks for a buffer. Any other code, such as
    // APPMODEL_ERROR_NO_PACKAGE (15700), is not packaged.
    #[allow(unsafe_code)]
    pub fn is_packaged(&self) -> bool {
        let mut length = 0u32;
        // SAFETY: `length` is a valid out pointer and the name buffer is null, which the call allows with a zero length.
        let code = unsafe { GetCurrentPackageFullName(&mut length, None) };
        code == ERROR_INSUFFICIENT_BUFFER
    }
}

impl Default for PackageContext {
    fn default() -> Self {
        Self::new()
    }
}
