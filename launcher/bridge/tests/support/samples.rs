use amml_gdk::{AmmlPaths, GameVersion, RegisteredPackage};
use amml_launcher_core::{AmmlState, InstalledMod, InstanceState, LauncherPaths, ModId, ModSource};
use amml_registry::AmmlRelease;
use std::path::{Path, PathBuf};

// Small values for the mapping tests.
pub struct Samples;

impl Samples {
    pub fn paths(root: &Path) -> LauncherPaths {
        LauncherPaths::new(AmmlPaths::new(root.to_path_buf()))
    }

    pub fn instance(root: &Path, name: &str) -> InstanceState {
        InstanceState {
            name: name.to_string(),
            folder: root.join("instances").join(name),
            game: GameVersion::parse("1.26.60.05").unwrap(),
            preview: false,
            amml: AmmlState { version: Some("26.60.1".to_string()), installed: true },
            mods: Vec::new(),
            data_dir: None,
            managed: true,
        }
    }

    pub fn installed(id: &str, version: Option<&str>, source: ModSource) -> InstalledMod {
        InstalledMod {
            id: ModId::parse(id).unwrap(),
            version: version.map(str::to_string),
            enabled: true,
            listing: None,
            api_version: 2,
            source,
            folder: PathBuf::from("mods"),
        }
    }

    pub fn package(family: &str, version: &str) -> RegisteredPackage {
        RegisteredPackage {
            family_name: family.to_string(),
            full_name: format!("{family}_{version}"),
            version: GameVersion::parse(version).unwrap(),
            status_ok: true,
            store_signed: true,
            install_location: PathBuf::from("C:\\Games\\Minecraft"),
            content_location: PathBuf::from("C:\\Games\\Minecraft\\Content"),
        }
    }

    pub fn amml_release(version: &str, games: &[&str]) -> AmmlRelease {
        AmmlRelease {
            version: version.to_string(),
            api_version: 2,
            game_versions: games.iter().map(|game| game.to_string()).collect(),
            asset: "amml.zip".to_string(),
            sha256: String::new(),
            size: 1,
        }
    }
}
