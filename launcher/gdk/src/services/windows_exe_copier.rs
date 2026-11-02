use super::{ExeCopier, PackageContextCommand, PeCheck, RobocopyLog};
use crate::GdkError;
use crate::structs::{ExeCopy, RegisteredPackage};
use std::io::Read;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct WindowsExeCopier {
    timeout: Duration,
}

impl WindowsExeCopier {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
    const POLL: Duration = Duration::from_millis(250);
    const SETTLE: Duration = Duration::from_secs(3);
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    pub fn new(timeout: Duration) -> WindowsExeCopier {
        WindowsExeCopier { timeout }
    }

    fn failed(method: &'static str, detail: impl Into<String>) -> GdkError {
        GdkError::ExeCopyFailed { method, detail: detail.into() }
    }

    fn source(package: &RegisteredPackage) -> PathBuf {
        package.install_location.join(PackageContextCommand::EXE_NAME)
    }

    // A plain process reads the exe of the version that is registered now; it gets Access denied on an older one.
    pub fn copy_plain(&self, package: &RegisteredPackage, dest_dir: &Path) -> Result<ExeCopy, GdkError> {
        let source = Self::source(package);
        let dest = dest_dir.join(PackageContextCommand::EXE_NAME);
        let expected = std::fs::metadata(&source).map_err(|error| Self::failed("plain", error.to_string()))?.len();
        if let Err(error) = std::fs::copy(&source, &dest) {
            let _ = std::fs::remove_file(&dest);
            return Err(Self::failed("plain", error.to_string()));
        }
        self.verify("plain", dest, expected)
    }

    // robocopy started with the game's package identity copies the exe; the cmdlet returns before robocopy ends
    pub fn copy_in_package_context(
        &self,
        package: &RegisteredPackage,
        app_id: &str,
        dest_dir: &Path,
    ) -> Result<ExeCopy, GdkError> {
        const METHOD: &str = "package-context";
        let source = Self::source(package);
        let expected = std::fs::metadata(&source).map_err(|error| Self::failed(METHOD, error.to_string()))?.len();
        std::fs::create_dir_all(dest_dir)?;
        let command = PackageContextCommand::new(&package.family_name, app_id, &package.install_location, dest_dir);
        let log = dest_dir.join(PackageContextCommand::LOG_NAME);
        let dest = dest_dir.join(PackageContextCommand::EXE_NAME);
        let _ = std::fs::remove_file(&log);
        let started = Instant::now();
        self.run_script(&command, started)?;
        let finished = self.wait_until_complete(&dest, &log, expected, started);
        let _ = std::fs::remove_file(&log);
        finished?;
        self.verify(METHOD, dest, expected)
    }

    // Runs the outer PowerShell. It returns when the cmdlet has started the copy, not when the copy is done.
    // A PowerShell that does not end in time is ended; it is this process's own child.
    fn run_script(&self, command: &PackageContextCommand, started: Instant) -> Result<(), GdkError> {
        const METHOD: &str = "package-context";
        let mut child = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-EncodedCommand",
                &command.encoded(),
            ])
            .envs(command.environment())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .creation_flags(Self::CREATE_NO_WINDOW)
            .spawn()
            .map_err(|error| Self::failed(METHOD, error.to_string()))?;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        return Ok(());
                    }
                    let mut text = String::new();
                    if let Some(mut stderr) = child.stderr.take() {
                        let _ = stderr.read_to_string(&mut text);
                    }
                    return Err(Self::failed(METHOD, text.trim().to_string()));
                }
                Ok(None) => {}
                Err(error) => return Err(Self::failed(METHOD, error.to_string())),
            }
            if started.elapsed() >= self.timeout {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Self::failed(METHOD, format!("PowerShell did not end in {} s", self.timeout.as_secs())));
            }
            std::thread::sleep(Self::POLL);
        }
    }

    // Done when the copy has the exact length and robocopy no longer holds it open. Failed when robocopy logged an
    // error and then wrote nothing for SETTLE: with one retry it has given up by then.
    fn wait_until_complete(&self, dest: &Path, log: &Path, expected: u64, started: Instant) -> Result<(), GdkError> {
        const METHOD: &str = "package-context";
        let mut last_log = (0u64, Instant::now());
        loop {
            let length = std::fs::metadata(dest).map(|found| found.len()).unwrap_or(0);
            if length == expected && std::fs::OpenOptions::new().read(true).share_mode(0).open(dest).is_ok() {
                return Ok(());
            }
            let log_length = std::fs::metadata(log).map(|found| found.len()).unwrap_or(0);
            if log_length != last_log.0 {
                last_log = (log_length, Instant::now());
            } else if last_log.1.elapsed() >= Self::SETTLE {
                let errors = RobocopyLog::errors(log);
                if !errors.is_empty() {
                    return Err(Self::failed(METHOD, format!("robocopy: {}", errors.join(" "))));
                }
            }
            if started.elapsed() >= self.timeout {
                let mut detail =
                    format!("the copy reached {length} of {expected} bytes in {} s", self.timeout.as_secs());
                let errors = RobocopyLog::errors(log);
                if !errors.is_empty() {
                    detail = format!("{detail}; robocopy: {}", errors.join(" "));
                }
                return Err(Self::failed(METHOD, detail));
            }
            std::thread::sleep(Self::POLL);
        }
    }

    fn verify(&self, method: &'static str, dest: PathBuf, expected: u64) -> Result<ExeCopy, GdkError> {
        let bytes = std::fs::metadata(&dest).map_err(|error| Self::failed(method, error.to_string()))?.len();
        if bytes != expected {
            let _ = std::fs::remove_file(&dest);
            return Err(Self::failed(method, format!("the copy has {bytes} bytes, the source has {expected}")));
        }
        if !PeCheck::is_x64(&dest)? {
            let _ = std::fs::remove_file(&dest);
            return Err(Self::failed(method, "the copy is not an x64 executable"));
        }
        Ok(ExeCopy { path: dest, bytes, method })
    }
}

impl ExeCopier for WindowsExeCopier {
    fn copy_exe(&self, package: &RegisteredPackage, app_id: &str, dest_dir: &Path) -> Result<ExeCopy, GdkError> {
        std::fs::create_dir_all(dest_dir)?;
        match self.copy_plain(package, dest_dir) {
            Ok(copy) => Ok(copy),
            Err(plain) => {
                curia::info!("plain exe copy refused; copying in the package context", { error: plain.to_string() });
                self.copy_in_package_context(package, app_id, dest_dir)
            }
        }
    }
}
