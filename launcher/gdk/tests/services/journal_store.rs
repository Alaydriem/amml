use amml_gdk::{Channel, GdkError, InstallJournal, JournalStore};
use std::path::PathBuf;

fn journal(staging: Option<PathBuf>) -> InstallJournal {
    InstallJournal {
        original_full_name: "Microsoft.MinecraftUWP_1.26.6005.0_x64__8wekyb3d8bbwe".to_string(),
        original_package: PathBuf::from(r"C:\Users\o'brien\AppData\Local\amml\launcher\downloads\a b.msixvc"),
        wanted_full_name: "Microsoft.MinecraftUWP_1.26.5203.0_x64__8wekyb3d8bbwe".to_string(),
        staging,
        channel: Channel::Release,
    }
}

#[test]
fn no_file_reads_as_no_journal() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(JournalStore::new(dir.path().join("j.hcl")).read().unwrap(), None);
}

#[test]
fn windows_paths_and_optional_staging_survive() {
    let dir = tempfile::tempdir().unwrap();
    let store = JournalStore::new(dir.path().join("launcher").join("j.hcl"));

    store.write(&journal(None)).unwrap();
    assert_eq!(store.read().unwrap(), Some(journal(None)));

    let staging = Some(PathBuf::from(r"C:\Users\x\AppData\Local\amml\versions\.staging-abc"));
    store.write(&journal(staging.clone())).unwrap();
    assert_eq!(store.read().unwrap(), Some(journal(staging)));
}

#[test]
fn write_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let store = JournalStore::new(dir.path().join("j.hcl"));
    store.write(&journal(None)).unwrap();
    let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
    assert_eq!(names, vec![std::ffi::OsString::from("j.hcl")]);
}

#[test]
fn clear_removes_the_file_and_can_repeat() {
    let dir = tempfile::tempdir().unwrap();
    let store = JournalStore::new(dir.path().join("j.hcl"));
    store.write(&journal(None)).unwrap();
    store.clear().unwrap();
    store.clear().unwrap();
    assert_eq!(store.read().unwrap(), None);
}

#[test]
fn garbage_is_a_bad_journal() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("j.hcl");
    std::fs::write(&path, "this is = = not a journal {").unwrap();
    assert!(matches!(JournalStore::new(path).read(), Err(GdkError::BadJournal(_))));
}

#[test]
fn an_empty_journal_is_a_bad_journal() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("j.hcl");
    std::fs::write(&path, b"").unwrap();
    assert!(matches!(JournalStore::new(path).read(), Err(GdkError::BadJournal(_))));
}
