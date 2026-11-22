use std::ffi::OsString;
use std::path::{Path, PathBuf};

use amml::HostPaths;

#[test]
fn the_levilamina_layout_hangs_off_the_folder_of_the_exe() {
    let paths = HostPaths::levilamina(Path::new("C:/game/Minecraft.Windows.exe"));

    assert_eq!(paths.game, Path::new("C:/game"));
    assert_eq!(paths.loader, Path::new("C:/game/mods/amml-loader"));
    assert_eq!(paths.recipes, Path::new("C:/game/mods/amml-loader/recipes"));
    assert_eq!(paths.cache, Path::new("C:/game/mods/amml-loader/cache"));
    assert_eq!(paths.data, Path::new("C:/game/mods/amml-loader/data"));
    assert_eq!(paths.settings, Path::new("C:/game/mods/amml-loader/amml.hcl"));
    assert_eq!(paths.logs, Path::new("C:/game/logs/amml"));
    assert_eq!(paths.mods, Path::new("C:/game/mods/amml"));
}

#[test]
fn the_mods_folder_is_not_one_levilamina_scans() {
    let paths = HostPaths::levilamina(Path::new("C:/game/Minecraft.Windows.exe"));

    assert_ne!(paths.mods, paths.loader);
    assert!(!paths.mods.starts_with(&paths.loader));
}

#[test]
fn the_standalone_layout_keeps_the_loader_files_in_one_amml_folder_beside_the_exe() {
    let paths = HostPaths::standalone(Path::new("C:/game/Minecraft.Windows.exe"));

    assert_eq!(paths.game, Path::new("C:/game"));
    assert_eq!(paths.loader, Path::new("C:/game/amml"));
    assert_eq!(paths.recipes, Path::new("C:/game/amml/recipes"));
    assert_eq!(paths.cache, Path::new("C:/game/amml/cache"));
    assert_eq!(paths.data, Path::new("C:/game/amml/data"));
    assert_eq!(paths.settings, Path::new("C:/game/amml/amml.hcl"));
    assert_eq!(paths.logs, Path::new("C:/game/amml/logs"));
    assert_eq!(paths.mods, Path::new("C:/game/mods/amml"));
}

#[test]
fn standalone_instance_file_is_in_loader_folder() {
    let paths = HostPaths::standalone(Path::new("C:/g/Minecraft.Windows.exe"));

    assert_eq!(paths.instance, Path::new("C:/g/amml/instance.hcl"));
}

#[test]
fn levilamina_instance_file_is_in_loader_folder() {
    let paths = HostPaths::levilamina(Path::new("C:/game/Minecraft.Windows.exe"));

    assert_eq!(paths.instance, Path::new("C:/game/mods/amml-loader/instance.hcl"));
}

#[test]
fn mod_data_root_is_amml_mod_data_under_local_app_data() {
    let root = HostPaths::mod_data_root(Some(OsString::from(r"C:\Users\x\AppData\Local")));

    assert_eq!(root, Some(PathBuf::from(r"C:\Users\x\AppData\Local\amml\mod-data")));
}

#[test]
fn mod_data_root_is_none_without_local_app_data() {
    assert_eq!(HostPaths::mod_data_root(None), None);
}

#[test]
fn loader_dirs_name_the_instance_and_the_redirected_data_folder() {
    let root = tempfile::tempdir().unwrap();
    let paths = HostPaths::standalone(&root.path().join("Minecraft.Windows.exe"));

    assert_eq!(paths.loader_dirs().instance, Some(root.path().to_path_buf()));
    assert_eq!(paths.loader_dirs().game_data, None);

    std::fs::create_dir_all(&paths.loader).unwrap();
    std::fs::write(&paths.instance, "data_dir = \"profile\"\n").unwrap();

    assert_eq!(paths.loader_dirs().game_data, Some(root.path().join("profile")));
}

#[test]
fn loader_dirs_share_mod_data_inside_the_redirected_data_folder() {
    let root = tempfile::tempdir().unwrap();
    let paths = HostPaths::standalone(&root.path().join("Minecraft.Windows.exe"));
    std::fs::create_dir_all(&paths.loader).unwrap();
    std::fs::write(&paths.instance, "data_dir = \"profile\"\n").unwrap();

    assert_eq!(paths.loader_dirs().mod_data, Some(root.path().join("profile").join("mod-data")));
}

#[test]
fn loader_dirs_keep_the_local_app_data_mod_data_without_a_data_dir() {
    let root = tempfile::tempdir().unwrap();
    let paths = HostPaths::standalone(&root.path().join("Minecraft.Windows.exe"));

    assert_eq!(paths.loader_dirs().mod_data, paths.mod_data);
}

#[test]
fn loader_dirs_keep_the_shared_mod_data_when_data_dir_is_the_play_data_folder() {
    let root = tempfile::tempdir().unwrap();
    let mut paths = HostPaths::standalone(&root.path().join("game").join("Minecraft.Windows.exe"));
    let local = root.path().join("Local");
    paths.mod_data = HostPaths::mod_data_root(Some(local.clone().into_os_string()));
    std::fs::create_dir_all(&paths.loader).unwrap();
    // The play data folder `%LOCALAPPDATA%\amml\data`, written with other separators and another case.
    let play = local.join("AMML").join("Data").to_string_lossy().replace('\\', "/");
    std::fs::write(&paths.instance, format!("data_dir = \"{play}\"\n")).unwrap();

    let dirs = paths.loader_dirs();

    assert_eq!(dirs.mod_data, Some(local.join("amml").join("mod-data")));
    assert!(dirs.game_data.is_some());
}
