use std::collections::BTreeSet;

use super::MaterialWrite;

/// What `MaterialPatch` keeps.
pub(crate) struct MaterialPatchState {
    /// The patched material and the bytes it had before the first patch.
    pub(crate) kept: Option<MaterialWrite>,
    /// A restore was asked for and is not written yet.
    pub(crate) restore_due: bool,
    /// The mods that drew with the patched material since it was patched.
    pub(crate) drawers: BTreeSet<usize>,
}
