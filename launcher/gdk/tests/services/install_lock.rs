use amml_gdk::{GdkError, InstallLock};

#[test]
fn a_held_lock_blocks_the_next_one_until_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("launcher").join("install.lock");

    let held = InstallLock::acquire(&path).unwrap();
    assert!(matches!(InstallLock::acquire(&path), Err(GdkError::Busy)));

    drop(held);
    assert!(InstallLock::acquire(&path).is_ok());
}
