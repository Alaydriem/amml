use std::sync::{Mutex, MutexGuard};

use crate::structs::{ProvideError, ProviderEntry};

/// Holds the versioned tables that mods offer to each other.
///
/// One owner can hold a name and major version, so at most one entry matches
/// a `require`.
pub struct ProviderRegistry<T: Clone> {
    entries: Mutex<Vec<ProviderEntry<T>>>,
}

impl<T: Clone> Default for ProviderRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> ProviderRegistry<T> {
    pub fn new() -> Self {
        Self { entries: Mutex::new(Vec::new()) }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<ProviderEntry<T>>> {
        self.entries.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Offers `table`. The owner that holds the name and major again replaces
    /// its table and minor; another owner gets `Taken`.
    pub fn provide(&self, name: &str, major: u16, minor: u16, owner: usize, table: T) -> Result<(), ProvideError> {
        let mut entries = self.lock();
        if let Some(entry) = entries.iter_mut().find(|entry| entry.name == name && entry.major == major) {
            if entry.owner != owner {
                return Err(ProvideError::Taken { owner: entry.owner });
            }
            entry.minor = minor;
            entry.table = table;
            return Ok(());
        }
        entries.push(ProviderEntry { name: name.to_string(), major, minor, owner, table });
        Ok(())
    }

    /// Gives the table with the same major and a minor of at least `min_minor`, with its minor.
    pub fn require(&self, name: &str, major: u16, min_minor: u16) -> Option<(T, u16)> {
        self.lock()
            .iter()
            .find(|entry| entry.name == name && entry.major == major && entry.minor >= min_minor)
            .map(|entry| (entry.table.clone(), entry.minor))
    }

    /// Removes every table of `owner`. Gives their names and majors.
    pub fn drop_owner(&self, owner: usize) -> Vec<(String, u16)> {
        let mut dropped = Vec::new();
        self.lock().retain(|entry| {
            if entry.owner == owner {
                dropped.push((entry.name.clone(), entry.major));
                false
            } else {
                true
            }
        });
        dropped
    }
}
