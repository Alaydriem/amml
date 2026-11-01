#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use amml_data::BuildIdReader;
use amml_patch::ImportPatcher;
use pelite::pe64::{Pe, PeFile};

#[path = "support/import_list.rs"]
mod import_list;

use import_list::ImportList;

struct Game;

impl Game {
    fn exe() -> PathBuf {
        match std::env::var("AMML_GAME_EXE") {
            Ok(path) => PathBuf::from(path),
            Err(error) => panic!("AMML_GAME_EXE: {error}"),
        }
    }
}

#[test]
#[ignore = "needs AMML_GAME_EXE, the pristine Minecraft.Windows.exe"]
fn the_real_exe_keeps_its_build_id_and_every_import_when_patched() {
    let input = std::fs::read(Game::exe()).unwrap();
    let work = tempfile::tempdir().unwrap();
    let output_path = work.path().join("Minecraft.Windows.exe");

    let output = ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").unwrap();
    std::fs::write(&output_path, &output).unwrap();

    assert_eq!(BuildIdReader::from_path(&Game::exe()).unwrap(), BuildIdReader::from_path(&output_path).unwrap());
    let before = ImportList::read(&input);
    let after = ImportList::read(&output);
    assert!(!before.is_empty());
    let patched = PeFile::from_bytes(&output).unwrap();
    assert_eq!(patched.data_directory()[4].VirtualAddress, 0);
    assert_eq!(patched.data_directory()[4].Size, 0);
    let original = PeFile::from_bytes(&input).unwrap();
    for descriptor in original.imports().unwrap() {
        assert_eq!(descriptor.image().TimeDateStamp, 0, "{}", descriptor.dll_name().unwrap());
    }
    assert_eq!(&after[..before.len()], &before[..]);
    assert_eq!(&after[before.len()..], ["amml.dll!amml_anchor"]);
}
