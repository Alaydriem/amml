use crate::GdkError;
use crate::structs::{ExeCopy, RegisteredPackage};
use std::path::Path;

pub trait ExeCopier {
    // copies `Minecraft.Windows.exe` of the registered package into `dest_dir` and checks the copy
    fn copy_exe(&self, package: &RegisteredPackage, app_id: &str, dest_dir: &Path) -> Result<ExeCopy, GdkError>;
}
