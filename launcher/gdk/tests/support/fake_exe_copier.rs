use super::PeBytes;
use amml_gdk::{ExeCopier, ExeCopy, GdkError, RegisteredPackage};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

// Writes a synthetic x64 executable, or fails the way a refused package-context copy does.
#[derive(Clone)]
pub struct FakeExeCopier {
    pub size: usize,
    pub failure: Option<String>,
    destinations: Rc<RefCell<Vec<PathBuf>>>,
}

impl FakeExeCopier {
    pub fn new(size: usize) -> FakeExeCopier {
        FakeExeCopier { size, failure: None, destinations: Rc::new(RefCell::new(Vec::new())) }
    }

    pub fn failing(detail: &str) -> FakeExeCopier {
        FakeExeCopier { failure: Some(detail.to_string()), ..FakeExeCopier::new(0) }
    }

    pub fn destinations(&self) -> Vec<PathBuf> {
        self.destinations.borrow().clone()
    }
}

impl ExeCopier for FakeExeCopier {
    fn copy_exe(&self, _package: &RegisteredPackage, _app_id: &str, dest_dir: &Path) -> Result<ExeCopy, GdkError> {
        self.destinations.borrow_mut().push(dest_dir.to_path_buf());
        if let Some(detail) = &self.failure {
            return Err(GdkError::ExeCopyFailed { method: "package-context", detail: detail.clone() });
        }
        std::fs::create_dir_all(dest_dir)?;
        let path = dest_dir.join("Minecraft.Windows.exe");
        std::fs::write(&path, PeBytes::x64(self.size))?;
        Ok(ExeCopy { path, bytes: self.size.max(0x86) as u64, method: "package-context" })
    }
}
