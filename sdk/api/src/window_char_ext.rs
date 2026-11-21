use amml_abi::WindowChar;

/// Typed reads of a `WindowChar`.
pub trait WindowCharExt {
    /// The character; `None` for a value that is not a Unicode scalar value (a lone surrogate).
    fn ch(&self) -> Option<char>;
}

impl WindowCharExt for WindowChar {
    fn ch(&self) -> Option<char> {
        char::from_u32(self.ch)
    }
}
