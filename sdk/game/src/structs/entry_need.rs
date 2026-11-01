/// Whether a mod can run without an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryNeed {
    /// The mod cannot run without the entry.
    Required,
    /// The named feature is off when the entry does not resolve.
    Optional { feature: &'static str },
}
