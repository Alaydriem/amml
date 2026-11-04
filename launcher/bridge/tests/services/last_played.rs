use amml_launcher_bridge::LastPlayed;
use std::fs::File;
use std::time::{Duration, SystemTime};

fn stamped(path: &std::path::Path, age_seconds: u64) -> SystemTime {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = File::create(path).unwrap();
    let time = SystemTime::now() - Duration::from_secs(age_seconds);
    file.set_modified(time).unwrap();
    time
}

#[test]
fn last_played_is_the_newest_log_file_time() {
    let dir = tempfile::tempdir().unwrap();
    let logs = dir.path().join("logs");
    stamped(&logs.join("old.log"), 5000);
    let newest = stamped(&logs.join("amm").join("session").join("new.log"), 60);
    stamped(&logs.join("amm").join("older.log"), 900);

    assert_eq!(LastPlayed::of(dir.path()), Some(newest));
}

#[test]
fn an_instance_without_logs_was_never_played() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(LastPlayed::of(dir.path()), None);
    std::fs::create_dir_all(dir.path().join("logs")).unwrap();
    assert_eq!(LastPlayed::of(dir.path()), None);
}

#[test]
fn files_deeper_than_three_levels_are_not_read() {
    let dir = tempfile::tempdir().unwrap();
    let shallow = stamped(&dir.path().join("logs").join("a.log"), 900);
    stamped(&dir.path().join("logs").join("b").join("c").join("d").join("deep.log"), 1);

    assert_eq!(LastPlayed::of(dir.path()), Some(shallow));
}
