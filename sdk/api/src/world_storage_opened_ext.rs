use std::path::Path;

use amml_abi::WorldStorageOpened;

use crate::lent_strings::LentStrings;

/// Typed reads of a `WorldStorageOpened`: the game opened the storage of a world.
pub trait WorldStorageOpenedExt {
    /// The folder of the storage. Valid only inside the listener call: a copy of the event, or one kept past its call,
    /// gives `None`, as do an empty text and a text that is not UTF-8.
    fn path(&self) -> Option<&Path>;
}

impl WorldStorageOpenedExt for WorldStorageOpened {
    fn path(&self) -> Option<&Path> {
        LentStrings::find(self, 0).map(Path::new)
    }
}
