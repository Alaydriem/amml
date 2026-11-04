use crate::{AmmlState, InstalledMod};
use amml_gdk::GameVersion;
use std::path::PathBuf;

// An instance as it is on disk right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceState {
    pub name: String,
    pub folder: PathBuf,
    pub game: GameVersion,
    pub preview: bool,
    pub amml: AmmlState,
    pub mods: Vec<InstalledMod>,
    pub data_dir: Option<PathBuf>,
    // `version.json` has a `launcher` block
    pub managed: bool,
}
