use std::collections::HashMap;

use amml_game::{EntryId, EntryNeed, EntryResolver, GameEntries};
use amml_sdk::SdkError;

/// A resolver that knows the ids it was given.
pub struct FakeEntries {
    known: HashMap<&'static str, u64>,
}

impl FakeEntries {
    /// Entries with exactly these values; every other id is missing.
    pub fn build(values: &[(EntryId, u64)]) -> GameEntries {
        let resolver = Self { known: values.iter().map(|(id, value)| (id.as_str(), *value)).collect() };
        let wanted: Vec<_> = values.iter().map(|(id, _)| (*id, EntryNeed::Optional { feature: "test" })).collect();
        GameEntries::resolve(&resolver, &wanted).unwrap()
    }
}

impl EntryResolver for FakeEntries {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        self.known.get(id).copied().ok_or_else(|| SdkError::MissingEntry(id.to_string()))
    }
}
