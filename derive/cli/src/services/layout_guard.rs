use amml_data::{Check, Entry, EntryId};

use crate::structs::LayoutRefusal;

/// The `entry_is` checks of an entry: they say which layout of the game the entry belongs to. The deriver runs them
/// with every other check (`CheckRunner`); the loader asks `refusal` to tell an entry that is not for this build apart
/// from a missing entry, from the values it resolved.
pub struct LayoutGuard;

impl LayoutGuard {
    /// True when the guard has a value and it is the one the check wants. A guard with no value never holds.
    pub fn holds(wanted: u64, found: Option<u64>) -> bool {
        found == Some(wanted)
    }

    /// The first `entry_is` check of `entry` whose guard has a value (from `value_of`) other than the check wants.
    ///
    /// The guard decides alone: such an entry is not for this build, whatever its recipes would give. A guard with
    /// no value is not a refusal, because the layout is then unknown, so the entry stays missing.
    pub fn refusal(entry: &Entry, value_of: impl Fn(&EntryId) -> Option<u64>) -> Option<LayoutRefusal> {
        entry.checks.iter().find_map(|check| match check {
            Check::EntryIs { entry: guard, value: wanted } => value_of(guard)
                .filter(|found| !Self::holds(*wanted, Some(*found)))
                .map(|found| LayoutRefusal { guard: guard.clone(), wanted: *wanted, found }),
            _ => None,
        })
    }

    /// The entries the `entry_is` checks of `entry` read.
    pub fn guards(entry: &Entry) -> impl Iterator<Item = &EntryId> {
        entry.checks.iter().filter_map(|check| match check {
            Check::EntryIs { entry: guard, .. } => Some(guard),
            _ => None,
        })
    }
}
