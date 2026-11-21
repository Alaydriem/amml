use std::path::{Path, PathBuf};
use std::process::Command;

/// Builds the events-mod fixture (a cdylib on `amml-abi` only) and reads the file its listeners write.
pub struct EventsMod;

impl EventsMod {
    // Each feature set gets its own target folder, so builds in parallel tests never overwrite each other.
    pub fn build(variant: &str, features: &[&str]) -> PathBuf {
        let target = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("events-mod-{variant}"));
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let mut command = Command::new(cargo);
        command
            .arg("build")
            .arg("-p")
            .arg("events-mod")
            .arg("--target-dir")
            .arg(&target)
            .current_dir(env!("CARGO_MANIFEST_DIR"));
        if !features.is_empty() {
            command.arg("--features").arg(features.join(","));
        }
        let status = command.status().unwrap();
        assert!(status.success(), "building the events-mod fixture failed");
        target.join("debug").join("events_mod.dll")
    }

    /// The lines every events-mod wrote to `<mods>/events.log`, in order.
    pub fn lines(mods: &Path) -> Vec<String> {
        std::fs::read_to_string(mods.join("events.log")).unwrap_or_default().lines().map(str::to_string).collect()
    }

    /// The lines of the mod `name` that start with `prefix` (after the name).
    pub fn lines_of(mods: &Path, name: &str, prefix: &str) -> Vec<String> {
        let head = format!("{name} {prefix}");
        Self::lines(mods).into_iter().filter(|line| line.starts_with(&head)).collect()
    }
}
