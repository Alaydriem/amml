use amml_gdk::DiskSpace;

#[test]
#[ignore = "needs a Windows drive"]
fn reads_free_space_of_the_temp_drive() {
    assert!(DiskSpace::new().free_bytes(&std::env::temp_dir()).unwrap() > 0);
}
