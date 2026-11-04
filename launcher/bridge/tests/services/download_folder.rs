use amml_launcher_bridge::{BridgeError, DownloadFileStateView, DownloadFolder};
use std::fs::File;
use std::path::Path;
use std::time::{Duration, SystemTime};

// writes a file of `size` bytes that is `age` seconds old
fn file(dir: &Path, name: &str, size: usize, age: u64) {
    std::fs::write(dir.join(name), vec![1u8; size]).unwrap();
    let handle = File::options().write(true).open(dir.join(name)).unwrap();
    handle.set_modified(SystemTime::now() - Duration::from_secs(age)).unwrap();
}

fn names(dir: &Path) -> Vec<String> {
    let mut found: Vec<String> =
        std::fs::read_dir(dir).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().to_string()).collect();
    found.sort();
    found
}

#[test]
fn removing_a_name_with_a_separator_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let outside = dir.path().join("outside.txt");
    std::fs::write(&outside, b"keep").unwrap();
    let inner = dir.path().join("downloads");
    std::fs::create_dir_all(&inner).unwrap();
    file(&inner, "current.msixvc", 5, 10);
    let folder = DownloadFolder::new(inner.clone(), vec!["current.msixvc".to_string()]);

    for id in ["..\\outside.txt", "../outside.txt", "sub/a.zip", "..", "", "current.msixvc"] {
        assert!(matches!(folder.remove(id), Err(BridgeError::Refused(_))), "{id:?}");
    }

    assert!(outside.is_file());
    assert!(inner.join("current.msixvc").is_file());
}

#[test]
fn removing_a_file_deletes_it_and_a_missing_file_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    file(dir.path(), "a.zip", 5, 10);
    file(dir.path(), "b.zip.part", 5, 10);
    file(dir.path(), "b.zip.part.meta", 5, 10);
    let folder = DownloadFolder::new(dir.path().to_path_buf(), Vec::new());

    folder.remove("a.zip").unwrap();
    folder.remove("b.zip.part").unwrap();

    assert!(names(dir.path()).is_empty());
    assert!(matches!(folder.remove("a.zip"), Err(BridgeError::Unavailable(_))));
}

#[test]
fn removing_the_unfinished_files_leaves_the_rest_and_counts_the_part_files() {
    let dir = tempfile::tempdir().unwrap();
    file(dir.path(), "a.zip", 5, 10);
    file(dir.path(), "b.zip.part", 5, 10);
    file(dir.path(), "b.zip.part.meta", 5, 10);
    file(dir.path(), "c.msixvc.part", 5, 10);
    let folder = DownloadFolder::new(dir.path().to_path_buf(), Vec::new());

    assert_eq!(folder.remove_unfinished().unwrap(), 2);

    assert_eq!(names(dir.path()), vec!["a.zip"]);
}

#[test]
fn the_view_names_each_file_state_and_counts_every_byte_used() {
    let dir = tempfile::tempdir().unwrap();
    file(dir.path(), "current.msixvc", 100, 10);
    file(dir.path(), "a.zip.part", 20, 10);
    file(dir.path(), "a.zip.part.meta", 3, 10);
    file(dir.path(), "b.zip", 7, 10);
    let folder = DownloadFolder::new(dir.path().to_path_buf(), vec!["current.msixvc".to_string()]);

    let view = folder.view();

    assert_eq!(view.used, 130);
    let states: Vec<(&str, DownloadFileStateView)> = view.files.iter().map(|f| (f.id.as_str(), f.state)).collect();
    assert_eq!(
        states,
        vec![
            ("a.zip.part", DownloadFileStateView::Unfinished),
            ("b.zip", DownloadFileStateView::Finished),
            ("current.msixvc", DownloadFileStateView::Kept),
        ]
    );
}

#[test]
fn a_kept_name_that_differs_in_case_from_the_file_is_still_kept() {
    let dir = tempfile::tempdir().unwrap();
    file(dir.path(), "Microsoft.MinecraftUWP_1.0.0.0.msixvc", 5, 10);
    let kept = vec!["microsoft.minecraftuwp_1.0.0.0.MSIXVC".to_string()];
    let folder = DownloadFolder::new(dir.path().to_path_buf(), kept);

    let refused = folder.remove("Microsoft.MinecraftUWP_1.0.0.0.msixvc");
    let shown = folder.view();

    assert!(matches!(refused, Err(BridgeError::Refused(_))));
    assert!(dir.path().join("Microsoft.MinecraftUWP_1.0.0.0.msixvc").is_file());
    assert_eq!(shown.files[0].state, DownloadFileStateView::Kept);
}

#[test]
fn a_name_typed_in_another_case_than_a_kept_file_is_refused_too() {
    let dir = tempfile::tempdir().unwrap();
    file(dir.path(), "current.msixvc", 5, 10);
    let folder = DownloadFolder::new(dir.path().to_path_buf(), vec!["current.msixvc".to_string()]);

    assert!(matches!(folder.remove("CURRENT.MSIXVC"), Err(BridgeError::Refused(_))));
    assert!(dir.path().join("current.msixvc").is_file());
}
