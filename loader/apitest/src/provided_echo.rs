use amml_api::Provided;

/// The table the API test mod provides as `apitest.echo` 1.0 and then requires from itself.
#[repr(C)]
pub struct ProvidedEcho {
    pub size: u32,
    pub version: u32,
    /// Gives `value + 1`, wrapping at `u32::MAX`.
    pub echo: extern "C" fn(u32) -> u32,
}

// SAFETY: `ProvidedEcho` is `#[repr(C)]` and starts with `size: u32, version: u32`, and `size` is its
// byte size.
#[allow(unsafe_code)]
unsafe impl Provided for ProvidedEcho {
    const NAME: &'static str = "apitest.echo";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

impl ProvidedEcho {
    /// The value `echo` gives for the probe the mod sends.
    pub const PROBE: u32 = 41;

    extern "C" fn echo(value: u32) -> u32 {
        value.wrapping_add(1)
    }
}

/// The one table the mod provides; a provided table must live for the process.
pub static ECHO: ProvidedEcho =
    ProvidedEcho { size: std::mem::size_of::<ProvidedEcho>() as u32, version: 1, echo: ProvidedEcho::echo };
