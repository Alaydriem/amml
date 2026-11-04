use amml_launcher_core::{DataChoice, InstanceHclFile};
use std::path::PathBuf;

#[test]
fn a_shared_path_is_written_so_the_loader_reads_it_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("amml").join("instance.hcl");
    let shared = PathBuf::from("C:\\Users\\o'brien\\AppData\\Local\\amml\\data");
    InstanceHclFile::write(&path, &DataChoice::Shared(shared.clone())).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\\\\Users\\\\o'brien"), "{text}");
    assert_eq!(InstanceHclFile::read_data_dir(&path, dir.path()), Some(shared));
    assert!(InstanceHclFile::exists(&path));
    assert!(!dir.path().join("amml").join("instance.hcl.tmp").exists());
}

#[test]
fn own_data_is_the_instance_folder() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("amml").join("instance.hcl");
    InstanceHclFile::write(&path, &DataChoice::Own).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "data_dir = \".\"\n");
    assert_eq!(InstanceHclFile::read_data_dir(&path, dir.path()), Some(dir.path().join(".")));
}

#[test]
fn a_missing_file_or_key_gives_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("instance.hcl");
    assert_eq!(InstanceHclFile::read_data_dir(&path, dir.path()), None);
    assert!(!InstanceHclFile::exists(&path));
    std::fs::write(&path, "other = 1\n").unwrap();
    assert_eq!(InstanceHclFile::read_data_dir(&path, dir.path()), None);
}
