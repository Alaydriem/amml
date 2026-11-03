use crate::CoreProgress;
use amml_gdk::DownloadProgress;

// Turns progress events into text lines. A download prints a line every 10 percent, not on every block.
#[derive(Default)]
pub struct CliProgress {
    last: Option<(String, u64)>,
}

impl CliProgress {
    pub fn new() -> CliProgress {
        CliProgress::default()
    }

    pub fn line(&mut self, event: &CoreProgress) -> Option<String> {
        match event {
            CoreProgress::InstallingVersion { step, download: None } => Some(step.label().to_string()),
            CoreProgress::InstallingVersion { step, download: Some(download) } => {
                self.download(step.label(), *download).map(|line| format!("  {line}"))
            }
            CoreProgress::CreatingInstance => Some("Creating the instance".to_string()),
            CoreProgress::Downloading { file, progress } => self.download(&format!("Downloading {file}"), *progress),
            CoreProgress::Verifying { file } => Some(format!("Checked {file}")),
            CoreProgress::InstallingAmml { version } => Some(format!("Installing AMML {version}")),
            CoreProgress::Patching => Some("Patching the game executable".to_string()),
            CoreProgress::InstallingMod { id } => Some(format!("Installing {id}")),
            CoreProgress::Finishing => Some("Finishing".to_string()),
            CoreProgress::Done => Some("Done".to_string()),
        }
    }

    // `None` while the tenth has not changed since the last line for this label
    pub fn download(&mut self, label: &str, progress: DownloadProgress) -> Option<String> {
        let tenth = progress.bytes_done * 10 / progress.bytes_total.max(1);
        let key = (label.to_string(), tenth);
        if self.last.as_ref() == Some(&key) {
            return None;
        }
        self.last = Some(key);
        Some(format!("{label} {}%", tenth * 10))
    }
}
