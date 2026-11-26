use std::path::PathBuf;

use amml_abi::{AmmlDirKind, AmmlStr, DirFn};

/// The folders of the mod. The loader decides where they are and creates none of them. Each is
/// `None` when the loader has no such folder.
#[derive(Clone, Copy)]
pub struct Dirs {
    dir: DirFn,
    module: &'static str,
}

impl Dirs {
    pub(crate) fn new(dir: DirFn, module: &'static str) -> Self {
        Self { dir, module }
    }

    /// The mod's own folder.
    pub fn mod_dir(&self) -> Option<PathBuf> {
        self.get(AmmlDirKind::MOD)
    }

    pub fn config_dir(&self) -> Option<PathBuf> {
        self.get(AmmlDirKind::CONFIG)
    }

    pub fn data_dir(&self) -> Option<PathBuf> {
        self.get(AmmlDirKind::DATA)
    }

    /// Outside the game folder; the same for every game instance.
    pub fn shared_data_dir(&self) -> Option<PathBuf> {
        self.get(AmmlDirKind::SHARED_DATA)
    }

    pub fn instance_dir(&self) -> Option<PathBuf> {
        self.get(AmmlDirKind::INSTANCE)
    }

    pub fn game_data_dir(&self) -> Option<PathBuf> {
        self.get(AmmlDirKind::GAME_DATA)
    }

    fn get(&self, kind: AmmlDirKind) -> Option<PathBuf> {
        let mut out = AmmlStr::EMPTY;
        // SAFETY: `module` lives for the call and `out` is a live AmmlStr.
        #[allow(unsafe_code)]
        let status = unsafe { (self.dir)(AmmlStr::from_str(self.module), kind, &mut out) };
        if !status.is_ok() {
            return None;
        }
        // SAFETY: the loader keeps the bytes valid for the process; this copies them.
        #[allow(unsafe_code)]
        let text = unsafe { out.as_str() }.ok()?;
        if text.is_empty() { None } else { Some(PathBuf::from(text)) }
    }
}
