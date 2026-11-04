use super::InstanceFolder;
use amml_gdk::{
    AmmlPaths, DownloadProgress, GdkError, InstallOutcome, InstallRequest, InstallStep, Installer, VersionFolder,
};
use std::cell::Cell;
use std::rc::Rc;

// Puts a plain game folder into `versions\<version>\` instead of talking to the Store. Clones share the counter.
#[derive(Clone)]
pub struct FakeInstaller {
    paths: AmmlPaths,
    calls: Rc<Cell<usize>>,
}

impl FakeInstaller {
    pub fn new(paths: AmmlPaths) -> FakeInstaller {
        FakeInstaller { paths, calls: Rc::new(Cell::new(0)) }
    }

    pub fn calls(&self) -> usize {
        self.calls.get()
    }
}

impl Installer for FakeInstaller {
    fn install(
        &self,
        request: &InstallRequest,
        on_step: &mut dyn FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<InstallOutcome, GdkError> {
        self.calls.set(self.calls.get() + 1);
        on_step(InstallStep::Checking, None);
        let path = self.paths.versions().join(request.version.instance_name());
        InstanceFolder::game_files(&path);
        Ok(InstallOutcome {
            version: VersionFolder { version: request.version, path, files: 0, bytes: 0 },
            path_used: "copy-out",
            final_full_name: String::new(),
            store_updated_meanwhile: false,
        })
    }

    fn recover(&self) -> Result<Option<String>, GdkError> {
        Ok(None)
    }
}
