use super::{PeBytes, Zips};
use amml_launcher_core::FileHash;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// A fake instance folder: the game files, a `version.json`, optionally AMML and mods.
pub struct InstanceFolder {
    pub dir: PathBuf,
}

impl InstanceFolder {
    // the plain game files of a version, as they sit in the version store
    pub fn game_files(dir: &Path) {
        std::fs::create_dir_all(dir.join("data").join("definitions")).unwrap();
        std::fs::write(dir.join("Minecraft.Windows.exe"), PeBytes::x64(8192)).unwrap();
        std::fs::write(dir.join("MicrosoftGame.Config"), b"<Game/>").unwrap();
        std::fs::write(dir.join("data").join("definitions").join("a.json"), b"{}").unwrap();
        std::fs::write(dir.join("b.txt"), b"b").unwrap();
    }

    // game files, `.bak` and a `version.json` for `game` (instance form, `1.26.60.05`)
    pub fn create(dir: &Path, game: &str) -> InstanceFolder {
        Self::game_files(dir);
        std::fs::copy(dir.join("Minecraft.Windows.exe"), dir.join("Minecraft.Windows.exe.bak")).unwrap();
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let text = format!(
            "{{\n  \"name\": \"{name}\",\n  \"gameVersion\": \"{game}\",\n  \"type\": \"release\",\n  \"enableIsolation\": true,\n  \"launchArgs\": \"\",\n  \"envVars\": \"\"\n}}\n"
        );
        std::fs::write(dir.join("version.json"), text).unwrap();
        InstanceFolder { dir: dir.to_path_buf() }
    }

    pub fn with_version_json(self, text: &str) -> InstanceFolder {
        std::fs::write(self.dir.join("version.json"), text).unwrap();
        self
    }

    // `amml.dll`, a recipe and the version file; `None` leaves the version file out
    pub fn with_amml(self, version: Option<&str>) -> InstanceFolder {
        std::fs::create_dir_all(self.dir.join("amml").join("recipes")).unwrap();
        std::fs::write(self.dir.join("amml.dll"), b"old loader").unwrap();
        std::fs::write(self.dir.join("amml").join("recipes").join("old.hcl"), b"old recipe").unwrap();
        if let Some(version) = version {
            std::fs::write(self.dir.join("amml").join("amml-version.txt"), format!("{version}\n")).unwrap();
        }
        self
    }

    // `ammlmml-api.txt`, as `install` writes it next to a local build
    pub fn with_amml_api(self, api: &str) -> InstanceFolder {
        std::fs::create_dir_all(self.dir.join("amml")).unwrap();
        std::fs::write(self.dir.join("amml").join("amml-api.txt"), api).unwrap();
        self
    }

    // a mod folder with an `amml.hcl`, a dll, `config\keys.hcl` and `data\db.bin`
    pub fn with_mod(
        self,
        enabled: bool,
        namespace: &str,
        name: &str,
        version: Option<&str>,
        api: u32,
    ) -> InstanceFolder {
        let side = if enabled { "amml" } else { "amml-disabled" };
        let folder = self.dir.join("mods").join(side).join(format!("@{namespace}")).join(name);
        std::fs::create_dir_all(folder.join("config")).unwrap();
        std::fs::create_dir_all(folder.join("data")).unwrap();
        std::fs::write(folder.join("amml.hcl"), Zips::manifest(namespace, name, version, api)).unwrap();
        std::fs::write(folder.join(format!("{name}.dll")), b"old dll").unwrap();
        std::fs::write(folder.join("config").join("keys.hcl"), b"old keys").unwrap();
        std::fs::write(folder.join("data").join("db.bin"), b"old db").unwrap();
        self
    }

    pub fn mod_folder(&self, enabled: bool, namespace: &str, name: &str) -> PathBuf {
        let side = if enabled { "amml" } else { "amml-disabled" };
        self.dir.join("mods").join(side).join(format!("@{namespace}")).join(name)
    }

    // every file below the instance: relative path to SHA-256
    pub fn hashes(dir: &Path) -> BTreeMap<String, String> {
        let mut found = BTreeMap::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(next) = stack.pop() {
            for entry in std::fs::read_dir(&next).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    stack.push(entry.path());
                } else {
                    let relative = entry.path().strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/");
                    found.insert(relative, FileHash::sha256(&entry.path()).unwrap());
                }
            }
        }
        found
    }

    // every folder name below `dir`, to find leftovers
    pub fn names(dir: &Path) -> Vec<String> {
        let mut found = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(next) = stack.pop() {
            for entry in std::fs::read_dir(&next).unwrap() {
                let entry = entry.unwrap();
                found.push(entry.file_name().to_string_lossy().to_string());
                if entry.file_type().unwrap().is_dir() {
                    stack.push(entry.path());
                }
            }
        }
        found
    }
}
