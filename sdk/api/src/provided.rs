/// A table one mod provides to others through the registry.
///
/// A provider adds fields only at the end and raises `MINOR` when it does; a new `MAJOR` is a new
/// table. A consumer gets a provider of the same major with a minor of at least its own.
///
/// # Safety
/// `Self` is `#[repr(C)]` and starts with `size: u32, version: u32`, where `size` is the byte size
/// of the provider's table. A consumer reads the provider's bytes as `Self`.
#[allow(unsafe_code)]
pub unsafe trait Provided: Sized + Sync + 'static {
    /// The registry name, for example `"zoom.api"`.
    const NAME: &'static str;
    const MAJOR: u16;
    const MINOR: u16;
}
