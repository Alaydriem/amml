use std::collections::BTreeMap;

/// The derived files of a registry, by path relative to the registry base.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PublishedTree {
    pub files: BTreeMap<String, Vec<u8>>,
}
