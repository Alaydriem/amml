use amml_launcher_core::{LauncherError, Patcher, ProcessPatcher};
use std::time::{Duration, Instant};

fn script(dir: &std::path::Path, body: &str) -> std::path::PathBuf {
    let path = dir.join("patch.cmd");
    std::fs::write(&path, format!("@echo off\r\n{body}\r\n")).unwrap();
    path
}

#[test]
fn exit_zero_is_success_and_the_arguments_are_passed() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.txt");
    let exe = script(dir.path(), "echo %1 %3 > \"%~4.args\"\r\ncopy \"%~2\" \"%~4\" >nul");
    let input = dir.path().join("in.exe");
    std::fs::write(&input, b"bytes").unwrap();
    ProcessPatcher::new(Duration::from_secs(20)).patch(&exe, &input, &out).unwrap();
    assert_eq!(std::fs::read(&out).unwrap(), b"bytes");
    let args = std::fs::read_to_string(dir.path().join("out.txt.args")).unwrap();
    assert!(args.contains("--input") && args.contains("--output"), "{args}");
}

#[test]
fn a_failing_patcher_reports_its_exit_code_and_last_error_line() {
    let dir = tempfile::tempdir().unwrap();
    let exe = script(dir.path(), "echo error: boom 1>&2\r\nexit /b 3");
    let result = ProcessPatcher::new(Duration::from_secs(20)).patch(&exe, &dir.path().join("a"), &dir.path().join("b"));
    match result {
        Err(LauncherError::PatchFailed { code, text }) => {
            assert_eq!(code, Some(3));
            assert_eq!(text, "error: boom");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_patcher_that_does_not_finish_is_ended_and_reported() {
    let dir = tempfile::tempdir().unwrap();
    let exe = script(
        dir.path(),
        ":again
goto again",
    );
    let started = Instant::now();
    let result =
        ProcessPatcher::new(Duration::from_millis(400)).patch(&exe, &dir.path().join("a"), &dir.path().join("b"));
    assert!(started.elapsed() < Duration::from_secs(4));
    assert!(matches!(result, Err(LauncherError::PatchFailed { code: None, .. })), "{result:?}");
}

#[test]
fn a_missing_patcher_is_an_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let result = ProcessPatcher::new(Duration::from_secs(5)).patch(
        &dir.path().join("missing.exe"),
        &dir.path().join("a"),
        &dir.path().join("b"),
    );
    assert!(matches!(result, Err(LauncherError::Io(_))));
}
