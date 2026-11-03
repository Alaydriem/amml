use amml_launcher_core::{LauncherError, Patcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// Writes `input` plus `PATCHED` to `output`, or fails. Clones share the call list.
#[derive(Clone, Default)]
pub struct FakePatcher {
    fail: bool,
    calls: Arc<Mutex<Vec<(PathBuf, PathBuf, PathBuf)>>>,
}

impl FakePatcher {
    pub fn new() -> FakePatcher {
        FakePatcher::default()
    }

    pub fn failing() -> FakePatcher {
        FakePatcher { fail: true, calls: Arc::default() }
    }

    pub fn calls(&self) -> Vec<(PathBuf, PathBuf, PathBuf)> {
        self.calls.lock().unwrap().clone()
    }
}

impl Patcher for FakePatcher {
    fn patch(&self, patcher_exe: &Path, input: &Path, output: &Path) -> Result<(), LauncherError> {
        self.calls.lock().unwrap().push((patcher_exe.to_path_buf(), input.to_path_buf(), output.to_path_buf()));
        if self.fail {
            return Err(LauncherError::PatchFailed { code: Some(1), text: "error: fake failure".to_string() });
        }
        let mut bytes = std::fs::read(input)?;
        bytes.extend_from_slice(b"PATCHED");
        std::fs::write(output, bytes)?;
        Ok(())
    }
}
