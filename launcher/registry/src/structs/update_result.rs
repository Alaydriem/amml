#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateResult {
    /// The release is new and the newest one: it owns the listing and the icon.
    Added,
    /// The release is new but older than a listed one: only the version list changed.
    Backfilled,
    /// Nothing changed: the release was already listed, or it is older than every version the registry keeps.
    Unchanged,
}
