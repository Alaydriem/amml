use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use amml_patch::cli::Cli;
use clap::Parser;
use tempfile::tempdir;

use crate::support::{ImportList, PeFixture};

struct Patcher;

impl Patcher {
    fn run(input: &Path, output: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_amml-patch"))
            .arg("--input")
            .arg(input)
            .arg("--output")
            .arg(output)
            .output()
            .unwrap()
    }

    fn temp_of(output: &Path) -> PathBuf {
        let mut name = output.as_os_str().to_os_string();
        name.push(".tmp");
        PathBuf::from(name)
    }
}

#[test]
fn failure_leaves_existing_output_untouched() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("not-a-pe.exe");
    let output = dir.path().join("patched.exe");
    std::fs::write(&input, b"this is not a PE file").unwrap();
    std::fs::write(&output, b"known bytes").unwrap();

    let result = Patcher::run(&input, &output);

    assert_eq!(result.status.code(), Some(1));
    assert_eq!(std::fs::read(&output).unwrap(), b"known bytes");
    assert!(!Patcher::temp_of(&output).exists());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.starts_with("error: "), "{stderr}");
    assert!(result.stdout.is_empty());
}

#[test]
fn failure_with_no_existing_output_writes_nothing() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("missing.exe");
    let output = dir.path().join("patched.exe");

    let result = Patcher::run(&input, &output);

    assert_eq!(result.status.code(), Some(1));
    assert!(!output.exists());
    assert!(!Patcher::temp_of(&output).exists());
}

#[test]
fn success_replaces_output_atomically() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("patched.exe");
    std::fs::write(&output, b"old bytes").unwrap();

    let result = Patcher::run(&PeFixture::small_path(), &output);

    assert_eq!(result.status.code(), Some(0), "{}", String::from_utf8_lossy(&result.stderr));
    let imports = ImportList::read(&std::fs::read(&output).unwrap());
    assert_eq!(imports.last().map(String::as_str), Some("amml.dll!amml_anchor"));
    assert!(!Patcher::temp_of(&output).exists());
}

#[test]
fn version_flag_prints_workspace_version() {
    let result = Command::new(env!("CARGO_BIN_EXE_amml-patch")).arg("--version").output().unwrap();

    assert_eq!(result.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&result.stdout).trim_end(), format!("amml-patch {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn the_documented_arguments_parse_with_their_defaults() {
    let cli = Cli::try_parse_from(["amml-patch", "--input", "a.exe", "--output", "b.exe"]).unwrap();

    assert_eq!(cli.dll, "amml.dll");
    assert_eq!(cli.symbol, "amml_anchor");
}

#[test]
fn a_bad_argument_gives_one_error_line_and_exit_1() {
    let result = Command::new(env!("CARGO_BIN_EXE_amml-patch")).arg("--no-such-flag").output().unwrap();

    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.starts_with("error: "), "{stderr}");
    assert!(stderr.contains("--no-such-flag"), "{stderr}");
}

#[test]
fn a_missing_required_argument_gives_one_error_line_and_exit_1() {
    let result = Command::new(env!("CARGO_BIN_EXE_amml-patch")).arg("--input").arg("a.exe").output().unwrap();

    assert_eq!(result.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("--output"), "{stderr}");
}

#[test]
fn help_exits_0_and_prints_to_stdout() {
    let result = Command::new(env!("CARGO_BIN_EXE_amml-patch")).arg("--help").output().unwrap();

    assert_eq!(result.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&result.stdout).contains("--input"));
    assert!(result.stderr.is_empty());
}
