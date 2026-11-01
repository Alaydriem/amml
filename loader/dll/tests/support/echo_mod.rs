use std::path::PathBuf;
use std::process::Command;

/// Builds the echo-mod fixture, a cdylib workspace member that shares the workspace lockfile, with a set of features.
pub struct EchoMod;

impl EchoMod {
    // Each feature set gets its own target folder, so builds in parallel tests never overwrite each other.
    pub fn build(variant: &str, features: &[&str]) -> PathBuf {
        let target = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("echo-mod-{variant}"));
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let mut command = Command::new(cargo);
        command
            .arg("build")
            .arg("-p")
            .arg("echo-mod")
            .arg("--target-dir")
            .arg(&target)
            .current_dir(env!("CARGO_MANIFEST_DIR"));
        if !features.is_empty() {
            command.arg("--features").arg(features.join(","));
        }
        let status = command.status().unwrap();
        assert!(status.success(), "building the echo-mod fixture failed");
        target.join("debug").join("echo_mod.dll")
    }
}
