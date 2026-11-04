use amml_gdk::{ProcessInspector, WindowsProcessInspector};

// Read-only: the desktop shell is always running in an interactive session.
#[test]
#[ignore = "needs an interactive Windows session"]
fn finds_the_running_shell_with_its_image_path() {
    let found = WindowsProcessInspector::new().image_paths("explorer.exe").unwrap();
    assert!(!found.is_empty());
    assert!(found.iter().all(|(_, path)| path.as_os_str().is_empty() || path.ends_with("explorer.exe")));
}
