use std::path::Path;
use std::time::SystemTime;

// When an instance was last played: AMML writes a log file for each session, so the newest file time under `logs\`.
pub struct LastPlayed;

impl LastPlayed {
    const DEPTH: u32 = 3;

    pub fn of(folder: &Path) -> Option<SystemTime> {
        Self::newest(&folder.join("logs"), Self::DEPTH)
    }

    fn newest(dir: &Path, depth: u32) -> Option<SystemTime> {
        let mut newest: Option<SystemTime> = None;
        for entry in std::fs::read_dir(dir).ok()?.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let found = if kind.is_dir() {
                if depth <= 1 { None } else { Self::newest(&entry.path(), depth - 1) }
            } else {
                entry.metadata().ok().and_then(|meta| meta.modified().ok())
            };
            newest = match (newest, found) {
                (Some(left), Some(right)) => Some(left.max(right)),
                (left, right) => left.or(right),
            };
        }
        newest
    }
}
