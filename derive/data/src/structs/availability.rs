use super::NeedKind;

/// Whether an event or area can run on a build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Availability {
    pub id: &'static str,
    pub kind: NeedKind,
    pub version: u32,
    /// The first shape whose entries all resolve.
    pub shape: Option<&'static str>,
    /// The entries of the first shape that did not resolve; empty when a shape holds.
    pub missing: Vec<&'static str>,
}
