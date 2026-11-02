use crate::GdkError;
use std::path::Path;

pub trait PackageDeployer {
    // registers the package file for the current user, over any registered version
    fn add(&self, package: &Path) -> Result<(), GdkError>;

    // Removes the registered package with this full name. A name in `protected` is the player's own Store
    // package: the deployer refuses it with `GdkError::Protected` without calling Windows.
    fn remove(&self, full_name: &str, protected: &[String]) -> Result<(), GdkError>;
}
