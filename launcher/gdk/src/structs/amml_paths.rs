use crate::GdkError;
use std::path::PathBuf;

// The folders the launcher owns, all under one root (`%LOCALAPPDATA%\amml` by default).
#[derive(Debug, Clone)]
pub struct AmmlPaths {
    root: PathBuf,
}

impl AmmlPaths {
    pub fn new(root: PathBuf) -> AmmlPaths {
        AmmlPaths { root }
    }

    pub fn default_root() -> Result<PathBuf, GdkError> {
        let base = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
            GdkError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "LOCALAPPDATA is not set"))
        })?;
        Ok(PathBuf::from(base).join("amml"))
    }

    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn versions(&self) -> PathBuf {
        self.root.join("versions")
    }

    pub fn instances(&self) -> PathBuf {
        self.root.join("instances")
    }

    pub fn downloads(&self) -> PathBuf {
        self.root.join("launcher").join("downloads")
    }

    pub fn journal(&self) -> PathBuf {
        self.root.join("launcher").join("install-journal.hcl")
    }

    pub fn lock(&self) -> PathBuf {
        self.root.join("launcher").join("install.lock")
    }

    pub fn logs(&self) -> PathBuf {
        self.root.join("launcher").join("logs")
    }
}
