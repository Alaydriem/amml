use crate::{GdkError, structs::RegisteredPackage};

pub trait PackageInventory {
    // the package registered for the current user in `family`, if any
    fn registered(&self, family: &str) -> Result<Option<RegisteredPackage>, GdkError>;
}
