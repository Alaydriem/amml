// A value read from a registry, with a flag for a value that came from the disk cache while the registry was down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cached<T> {
    pub value: T,
    pub stale: bool,
}
