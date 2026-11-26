use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{InstanceSettings, LoaderDirs};

/// Where the loader keeps its files, derived from the game executable.
///
/// In the standalone host the loader folder is `<game>/amml/` (`recipes/`, `cache/`, `data/` and the
/// optional `amml.hcl` and `instance.hcl`), logs go to `<game>/amml/logs/`, and AMML mods live in
/// `<game>/mods/amml/<name>/`.
///
/// In the LeviLamina host the loader package is `<game>/mods/amml-loader/` (the DLL, `recipes/`,
/// `cache/`, `data/` and the optional `amml.hcl` and `instance.hcl`), logs go to `<game>/logs/amml/`, and AMML mods live in
/// `<game>/mods/amml/<name>/` (with `config/` and `data/` inside) so LeviLamina does not try to load
/// them itself.
///
/// In both hosts the shared data root is `%LOCALAPPDATA%\amml\mod-data`, outside the game folder, so a
/// mod can keep files that every game instance reads. It is `None` when `LOCALAPPDATA` is not set. An instance whose
/// `instance.hcl` sets a `data_dir` other than the play data folder `%LOCALAPPDATA%\amml\data` (a test instance) gets
/// `<data_dir>\mod-data` instead (`loader_dirs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostPaths {
    pub exe: PathBuf,
    pub game: PathBuf,
    pub loader: PathBuf,
    pub recipes: PathBuf,
    pub cache: PathBuf,
    pub data: PathBuf,
    pub settings: PathBuf,
    pub instance: PathBuf,
    pub logs: PathBuf,
    pub mods: PathBuf,
    pub mod_data: Option<PathBuf>,
}

impl HostPaths {
    pub const LOADER_FOLDER: &'static str = "amml-loader";
    pub const STANDALONE_FOLDER: &'static str = "amml";
    pub const MODS_FOLDER: &'static str = "amml";
    pub const MOD_DATA_FOLDER: &'static str = "mod-data";
    pub const PLAY_DATA_FOLDER: &'static str = "data";
    pub const SETTINGS_FILE: &'static str = "amml.hcl";
    pub const INSTANCE_FILE: &'static str = "instance.hcl";

    /// The layout for a game executable run by LeviLamina. `<game>` is the folder of `exe_path`.
    pub fn levilamina(exe_path: &Path) -> Self {
        let game = exe_path.parent().map(Path::to_path_buf).unwrap_or_default();
        let mods_root = game.join("mods");
        let loader = mods_root.join(Self::LOADER_FOLDER);
        Self {
            exe: exe_path.to_path_buf(),
            recipes: loader.join("recipes"),
            cache: loader.join("cache"),
            data: loader.join("data"),
            settings: loader.join(Self::SETTINGS_FILE),
            instance: loader.join(Self::INSTANCE_FILE),
            logs: game.join("logs").join("amml"),
            mods: mods_root.join(Self::MODS_FOLDER),
            mod_data: Self::mod_data_root(std::env::var_os("LOCALAPPDATA")),
            loader,
            game,
        }
    }

    /// The layout for a game executable that AMML starts itself. `<game>` is the folder of `exe_path`.
    pub fn standalone(exe_path: &Path) -> Self {
        let game = exe_path.parent().map(Path::to_path_buf).unwrap_or_default();
        let loader = game.join(Self::STANDALONE_FOLDER);
        Self {
            exe: exe_path.to_path_buf(),
            recipes: loader.join("recipes"),
            cache: loader.join("cache"),
            data: loader.join("data"),
            settings: loader.join(Self::SETTINGS_FILE),
            instance: loader.join(Self::INSTANCE_FILE),
            logs: loader.join("logs"),
            mods: game.join("mods").join(Self::MODS_FOLDER),
            mod_data: Self::mod_data_root(std::env::var_os("LOCALAPPDATA")),
            loader,
            game,
        }
    }

    /// `<local_app_data>/amml/mod-data`, or `None` when `local_app_data` is `None`.
    pub fn mod_data_root(local_app_data: Option<OsString>) -> Option<PathBuf> {
        local_app_data.map(|root| PathBuf::from(root).join(Self::MODS_FOLDER).join(Self::MOD_DATA_FOLDER))
    }

    /// The folders handed to `LoaderState`: the instance folder is `<game>`, and the game data folder
    /// is the `data_dir` of `instance.hcl` when it sets one (an unreadable file sets none). The shared mod data root is
    /// `<data_dir>\mod-data` for a `data_dir` that is not the play data folder (the sibling `data` of the shared root,
    /// `%LOCALAPPDATA%\amml\data`, compared without case and separator differences); otherwise the shared root.
    pub fn loader_dirs(&self) -> LoaderDirs {
        let game_data = InstanceSettings::load(&self.instance, &self.game).ok().and_then(|settings| settings.data_dir);
        let mod_data = match &game_data {
            Some(data_dir) if !self.is_play_data(data_dir) => Some(data_dir.join(Self::MOD_DATA_FOLDER)),
            _ => self.mod_data.clone(),
        };
        let mut dirs = LoaderDirs::new(self.mods.clone(), self.logs.clone(), mod_data);
        dirs.instance = Some(self.game.clone());
        dirs.game_data = game_data;
        dirs
    }

    // Whether `data_dir` is the play data folder, the `data` folder beside the shared mod data root.
    fn is_play_data(&self, data_dir: &Path) -> bool {
        let Some(play) = self.mod_data.as_deref().and_then(Path::parent).map(|amml| amml.join(Self::PLAY_DATA_FOLDER))
        else {
            return false;
        };
        let text = |path: &Path| path.to_string_lossy().replace('/', "\\").trim_end_matches('\\').to_lowercase();
        text(&play) == text(data_dir)
    }
}
