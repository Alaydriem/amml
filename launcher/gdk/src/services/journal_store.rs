use crate::GdkError;
use crate::structs::InstallJournal;
use std::io::Write;
use std::path::PathBuf;

pub struct JournalStore {
    path: PathBuf,
}

impl JournalStore {
    pub fn new(path: PathBuf) -> JournalStore {
        JournalStore { path }
    }

    // a temp file and a rename, so a reader never sees half a journal
    pub fn write(&self, journal: &InstallJournal) -> Result<(), GdkError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = hcl::to_string(journal).map_err(|error| GdkError::BadJournal(error.to_string()))?;
        let temp = self.path.with_extension("hcl.tmp");
        // the data reaches the disk before the rename makes it the journal
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp, &self.path)?;
        Ok(())
    }

    pub fn read(&self) -> Result<Option<InstallJournal>, GdkError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        hcl::from_str(&text).map(Some).map_err(|error| GdkError::BadJournal(error.to_string()))
    }

    pub fn clear(&self) -> Result<(), GdkError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}
