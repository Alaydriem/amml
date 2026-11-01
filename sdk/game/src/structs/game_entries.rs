use std::collections::HashMap;

use amml_sdk::SdkError;

use crate::{EntryId, EntryNeed, EntryResolver, GameError};

/// The entries a mod asked for, resolved once, with the features whose optional entries are missing.
#[derive(Debug)]
pub struct GameEntries {
    values: HashMap<EntryId, u64>,
    off: Vec<(&'static str, String)>,
}

impl GameEntries {
    /// Resolves every wanted entry. An optional entry that fails for any reason turns only its feature off; a
    /// required entry that fails is an error.
    pub fn resolve<R: EntryResolver>(resolver: &R, wanted: &[(EntryId, EntryNeed)]) -> Result<Self, GameError> {
        let mut values = HashMap::new();
        let mut off: Vec<(&'static str, String)> = Vec::new();
        for (id, need) in wanted {
            match (resolver.resolve(id.as_str()), need) {
                (Ok(value), _) => {
                    values.insert(*id, value);
                }
                (Err(error), EntryNeed::Optional { feature }) => {
                    if !off.iter().any(|(name, _)| name == feature) {
                        off.push((feature, format!("{}: {error}", id.as_str())));
                    }
                }
                (Err(SdkError::MissingEntry(_)), EntryNeed::Required) => {
                    return Err(GameError::MissingEntry { id: id.as_str().to_string() });
                }
                (Err(source), EntryNeed::Required) => {
                    return Err(GameError::Resolve { id: id.as_str().to_string(), source });
                }
            }
        }
        Ok(Self { values, off })
    }

    /// The resolved value; `None` for an optional entry that did not resolve and for an id nobody asked for.
    pub fn get(&self, id: EntryId) -> Option<u64> {
        self.values.get(&id).copied()
    }

    /// The features that are off, each once, in the order they were found, with the entry id and the cause of the
    /// first failure that turned the feature off.
    pub fn off_features(&self) -> Vec<(&'static str, String)> {
        self.off.clone()
    }
}
