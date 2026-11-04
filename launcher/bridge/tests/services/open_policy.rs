use amml_launcher_bridge::{BridgeError, OpenPolicy};

#[test]
fn a_folder_inside_a_root_may_be_opened() {
    let dir = tempfile::tempdir().unwrap();
    let inner = dir.path().join("instances").join("play");
    std::fs::create_dir_all(&inner).unwrap();

    let opened = OpenPolicy::path(&[dir.path().to_path_buf()], inner.to_str().unwrap()).unwrap();

    // the path is handed on as it was given, not as the long \\?\ form
    assert_eq!(opened, inner);
}

#[test]
fn a_path_outside_every_root_or_leading_out_by_dots_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("amml");
    let neighbour = dir.path().join("amml-other");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&neighbour).unwrap();
    let roots = [root.clone()];

    let escaped = format!(r"{}\..\amml-other", root.display());
    assert!(matches!(OpenPolicy::path(&roots, &escaped), Err(BridgeError::Refused(_))));
    assert!(matches!(OpenPolicy::path(&roots, neighbour.to_str().unwrap()), Err(BridgeError::Refused(_))));
    assert!(matches!(OpenPolicy::path(&roots, r"C:\Windows"), Err(BridgeError::Refused(_))));
    assert!(matches!(OpenPolicy::path(&roots, root.join("missing").to_str().unwrap()), Err(BridgeError::Refused(_))));
}

#[test]
fn only_web_and_store_addresses_without_spaces_may_be_opened() {
    assert!(OpenPolicy::url("https://github.com/Alaydriem").is_ok());
    assert!(OpenPolicy::url("ms-windows-store://pdp/?ProductId=9NBLGGH2JHXJ").is_ok());
    assert!(OpenPolicy::url("HTTPS://example.org/").is_ok());
    for refused in [
        "http://example.org/",
        "file:///C:/Windows/System32/cmd.exe",
        "javascript:alert(1)",
        r"C:\Windows\notepad.exe",
        "https://example.org/a b",
        "https://example.org/\u{7}",
        "",
    ] {
        assert!(matches!(OpenPolicy::url(refused), Err(BridgeError::Refused(_))), "{refused:?}");
    }
}

#[test]
fn a_file_inside_a_root_is_refused_because_opening_it_would_run_it() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("mods").join("run.bat");
    std::fs::create_dir_all(script.parent().unwrap()).unwrap();
    std::fs::write(&script, b"echo hi").unwrap();

    let refused = OpenPolicy::path(&[dir.path().to_path_buf()], script.to_str().unwrap());

    assert!(matches!(refused, Err(BridgeError::Refused(_))));
}
