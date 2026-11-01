/// Rules for the two-field mod id `@<namespace>/<name>`.
pub struct ModId;

impl ModId {
    /// The namespace and name that only the loader itself may use.
    pub const RESERVED: (&'static str, &'static str) = ("amml", "loader");

    /// The mod id: `@<namespace>/<name>`.
    pub fn format(namespace: &str, name: &str) -> String {
        format!("@{namespace}/{name}")
    }

    /// True for a non-empty segment of `A-Z a-z 0-9 _ . -` that does not start with a dot. A
    /// segment is used as a folder or file name, so it can never hold a path separator.
    pub fn is_valid_segment(text: &str) -> bool {
        !text.is_empty()
            && !text.starts_with('.')
            && text.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
    }

    /// True when the pair is `@amml/loader`, in any case.
    pub fn is_reserved(namespace: &str, name: &str) -> bool {
        namespace.eq_ignore_ascii_case(Self::RESERVED.0) && name.eq_ignore_ascii_case(Self::RESERVED.1)
    }
}
