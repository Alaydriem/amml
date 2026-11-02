use crate::GdkError;
use std::path::PathBuf;

pub trait ProcessInspector {
    // the process id and full image path of every running process whose image file is named `exe_name`.
    // A process whose path cannot be read comes back with an empty path.
    fn image_paths(&self, exe_name: &str) -> Result<Vec<(u32, PathBuf)>, GdkError>;
}
