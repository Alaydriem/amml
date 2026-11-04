use super::Patcher;
use crate::LauncherError;
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// Runs `amml-patch.exe --input <in> --output <out>` with no window and waits for it.
pub struct ProcessPatcher {
    timeout: Duration,
}

impl ProcessPatcher {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const POLL: Duration = Duration::from_millis(20);
    const STDERR_LIMIT: u64 = 8 * 1024;

    pub fn new(timeout: Duration) -> ProcessPatcher {
        ProcessPatcher { timeout }
    }
}

impl Patcher for ProcessPatcher {
    fn patch(&self, patcher_exe: &Path, input: &Path, output: &Path) -> Result<(), LauncherError> {
        let mut child = Command::new(patcher_exe)
            .arg("--input")
            .arg(input)
            .arg("--output")
            .arg(output)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .creation_flags(Self::CREATE_NO_WINDOW)
            .spawn()?;
        let stderr = child.stderr.take().map(|mut pipe| {
            std::thread::spawn(move || {
                let mut text = String::new();
                let _ = pipe.by_ref().take(Self::STDERR_LIMIT).read_to_string(&mut text);
                text
            })
        });
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if started.elapsed() >= self.timeout {
                let _ = child.kill();
                let _ = child.wait();
                return Err(LauncherError::PatchFailed {
                    code: None,
                    text: format!("the patcher did not finish in {} seconds", self.timeout.as_secs()),
                });
            }
            std::thread::sleep(Self::POLL);
        };
        let text = stderr.and_then(|reader| reader.join().ok()).unwrap_or_default();
        if status.success() {
            return Ok(());
        }
        let line = text.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or_default().trim().to_string();
        Err(LauncherError::PatchFailed { code: status.code(), text: line })
    }
}
