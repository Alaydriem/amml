// The answer to a conditional GET.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fetched {
    NotModified,
    Body { bytes: Vec<u8>, etag: Option<String> },
}
