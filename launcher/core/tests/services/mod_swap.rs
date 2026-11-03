use amml_launcher_core::ModSwap;
use std::os::windows::fs::OpenOptionsExt;

#[test]
fn a_failed_restore_keeps_staging_with_the_players_config() {
    let root = tempfile::tempdir().unwrap();
    let existing = root.path().join("mods").join("@a").join("b");
    std::fs::create_dir_all(existing.join("config")).unwrap();
    std::fs::create_dir_all(existing.join("data")).unwrap();
    std::fs::write(existing.join("config").join("keys.hcl"), b"player keys").unwrap();
    std::fs::write(existing.join("data").join("db.bin"), b"player db").unwrap();
    std::fs::write(existing.join("b.dll"), b"dll").unwrap();
    let staging = root.path().join("mods").join(".staging-x");
    std::fs::create_dir_all(&staging).unwrap();
    std::fs::write(staging.join("new.dll"), b"new").unwrap();
    let aside = root.path().join("mods").join(".old-x");

    let mut swap = ModSwap::new(staging.clone(), aside, Some(existing.clone()));
    // an open file stops the old folder from moving aside, after config and data moved into staging
    let held = std::fs::OpenOptions::new().read(true).share_mode(0).open(existing.join("b.dll")).unwrap();
    assert!(swap.run(&existing).is_err());
    drop(held);
    // a folder in the way makes the restore of `config` fail
    std::fs::create_dir_all(existing.join("config")).unwrap();
    std::fs::write(existing.join("config").join("other"), b"x").unwrap();

    swap.undo(&existing);

    assert_eq!(std::fs::read(staging.join("config").join("keys.hcl")).unwrap(), b"player keys");
    assert_eq!(std::fs::read(existing.join("data").join("db.bin")).unwrap(), b"player db");
}

#[test]
fn a_clean_undo_removes_staging() {
    let root = tempfile::tempdir().unwrap();
    let existing = root.path().join("mods").join("@a").join("b");
    std::fs::create_dir_all(existing.join("config")).unwrap();
    std::fs::write(existing.join("config").join("keys.hcl"), b"player keys").unwrap();
    std::fs::write(existing.join("b.dll"), b"dll").unwrap();
    let staging = root.path().join("mods").join(".staging-x");
    std::fs::create_dir_all(&staging).unwrap();
    let mut swap = ModSwap::new(staging.clone(), root.path().join("mods").join(".old-x"), Some(existing.clone()));
    let held = std::fs::OpenOptions::new().read(true).share_mode(0).open(existing.join("b.dll")).unwrap();
    assert!(swap.run(&existing).is_err());
    drop(held);
    swap.undo(&existing);
    assert!(!staging.exists());
    assert_eq!(std::fs::read(existing.join("config").join("keys.hcl")).unwrap(), b"player keys");
}
