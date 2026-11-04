use amml_launcher_core::MsiHold;
use std::fs::OpenOptions;

#[test]
fn while_the_file_is_held_nothing_can_write_replace_or_delete_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("setup.msi");
    std::fs::write(&path, b"installer").unwrap();
    let hold = MsiHold::open(&path).unwrap();

    assert!(OpenOptions::new().write(true).open(&path).is_err());
    assert!(std::fs::remove_file(&path).is_err());
    let other = dir.path().join("other.msi");
    std::fs::write(&other, b"swap").unwrap();
    assert!(std::fs::rename(&other, &path).is_err());
    assert!(std::fs::read(&path).is_ok());

    drop(hold);
    assert!(OpenOptions::new().write(true).open(&path).is_ok());
}

#[test]
fn the_hash_is_read_through_the_held_handle_and_can_be_asked_twice() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("setup.msi");
    std::fs::write(&path, b"abc").unwrap();
    let mut hold = MsiHold::open(&path).unwrap();

    let first = hold.sha256().unwrap();
    let second = hold.sha256().unwrap();

    assert_eq!(first, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(first, second);
}
