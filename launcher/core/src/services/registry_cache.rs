use crate::{CacheLock, CachedFile, LauncherError, RegistryRef};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

// The disk cache of registry files: `<dir>\<cache key>\<relative>` and `<relative>.etag`.
// The total size of every registry's files stays within `budget`; the least recently read file goes first.
pub struct RegistryCache {
    dir: PathBuf,
    budget: u64,
    // the last time stamp handed out; file times only move forward, so eviction order is exact
    clock: Mutex<SystemTime>,
    // shared by every cache on this folder; held for a whole read, write or evict
    lock: CacheLock,
}

impl RegistryCache {
    pub const BUDGET: u64 = 50 * 1024 * 1024;
    const ETAG_SUFFIX: &'static str = ".etag";
    const TEMP_SUFFIX: &'static str = ".tmp";

    pub fn new(dir: PathBuf, budget: u64, lock: CacheLock) -> RegistryCache {
        RegistryCache { dir, budget, clock: Mutex::new(SystemTime::UNIX_EPOCH), lock }
    }

    // only `[A-Za-z0-9@._/-]`, no empty, `.` or `..` segment and no leading `/`
    pub fn safe_relative(relative: &str) -> Result<&str, LauncherError> {
        let allowed =
            |character: char| character.is_ascii_alphanumeric() || matches!(character, '@' | '.' | '_' | '/' | '-');
        let valid = !relative.is_empty()
            && relative.chars().all(allowed)
            && relative.split('/').all(|segment| {
                !segment.is_empty() && segment != "." && segment != ".." && !Self::reserved_segment(segment)
            });
        if valid { Ok(relative) } else { Err(LauncherError::BadModId(relative.to_string())) }
    }

    pub fn read(&self, registry: &RegistryRef, relative: &str) -> Option<(Vec<u8>, Option<String>)> {
        let path = self.path(registry, relative).ok()?;
        let _held = self.lock.hold();
        let bytes = std::fs::read(&path).ok()?;
        let etag_path = Self::etag_path(&path);
        let etag = std::fs::read_to_string(&etag_path).ok().filter(|text| !text.is_empty());
        self.touch(&path);
        self.touch(&etag_path);
        Some((bytes, etag))
    }

    pub fn write(
        &self,
        registry: &RegistryRef,
        relative: &str,
        bytes: &[u8],
        etag: Option<&str>,
    ) -> Result<(), LauncherError> {
        let path = self.path(registry, relative)?;
        let _held = self.lock.hold();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let etag_path = Self::etag_path(&path);
        Self::replace_file(&path, bytes)?;
        self.touch(&path);
        match etag {
            Some(etag) => {
                Self::replace_file(&etag_path, etag.as_bytes())?;
                self.touch(&etag_path);
            }
            None => {
                let _ = std::fs::remove_file(&etag_path);
            }
        }
        self.evict(&path);
        Ok(())
    }

    // `.tmp` beside the file, then a rename, so a crash never leaves a cut file
    fn replace_file(path: &Path, bytes: &[u8]) -> Result<(), LauncherError> {
        let temporary = path.with_file_name(format!(
            "{}{}",
            path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default(),
            Self::TEMP_SUFFIX
        ));
        std::fs::write(&temporary, bytes)?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    }

    // a segment Windows treats as a device, one that ends in a dot or space, or one the cache uses for its own files
    fn reserved_segment(segment: &str) -> bool {
        let stem = segment.split('.').next().unwrap_or_default().to_ascii_uppercase();
        let numbered = |prefix: &str| {
            stem.strip_prefix(prefix).is_some_and(|rest| rest.len() == 1 && matches!(rest.as_bytes()[0], b'1'..=b'9'))
        };
        matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || numbered("COM")
            || numbered("LPT")
            || segment.ends_with('.')
            || segment.ends_with(' ')
            || segment.ends_with(Self::ETAG_SUFFIX)
            || segment.ends_with(Self::TEMP_SUFFIX)
    }

    fn path(&self, registry: &RegistryRef, relative: &str) -> Result<PathBuf, LauncherError> {
        let safe = Self::safe_relative(relative)?;
        let mut path = self.dir.join(registry.cache_key());
        for segment in safe.split('/') {
            path.push(segment);
        }
        Ok(path)
    }

    fn etag_path(path: &Path) -> PathBuf {
        let mut name = path.as_os_str().to_os_string();
        name.push(Self::ETAG_SUFFIX);
        PathBuf::from(name)
    }

    fn stamp(&self) -> SystemTime {
        let mut last = self.clock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = SystemTime::now();
        let next = if now > *last { now } else { *last + Duration::from_micros(1) };
        *last = next;
        next
    }

    fn touch(&self, path: &Path) {
        if let Ok(file) = OpenOptions::new().write(true).open(path) {
            let _ = file.set_modified(self.stamp());
        }
    }

    // Removes the oldest files until the total is within the budget. The file just written stays.
    fn evict(&self, keep: &Path) {
        let mut files = Vec::new();
        Self::collect(&self.dir, &mut files);
        let mut total: u64 = files.iter().map(|file| file.size).sum();
        if total <= self.budget {
            return;
        }
        files.retain(|file| file.path != keep);
        files.sort_by_key(|file| file.modified);
        for file in files {
            if total <= self.budget {
                break;
            }
            if std::fs::remove_file(&file.path).is_ok() {
                total = total.saturating_sub(file.size);
                let etag_path = Self::etag_path(&file.path);
                if let Ok(metadata) = std::fs::metadata(&etag_path)
                    && std::fs::remove_file(&etag_path).is_ok()
                {
                    total = total.saturating_sub(metadata.len());
                }
            }
        }
    }

    // every data file under `dir` with the size of the file and of its etag file
    fn collect(dir: &Path, found: &mut Vec<CachedFile>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                Self::collect(&path, found);
            } else if !path.to_string_lossy().ends_with(Self::ETAG_SUFFIX) {
                let Ok(metadata) = entry.metadata() else {
                    continue;
                };
                let etag_size = std::fs::metadata(Self::etag_path(&path)).map(|meta| meta.len()).unwrap_or(0);
                found.push(CachedFile {
                    path,
                    size: metadata.len() + etag_size,
                    modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                });
            }
        }
    }
}
