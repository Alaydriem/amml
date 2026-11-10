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
