use crate::LauncherError;
use amml_gdk::ProcessInspector;
use std::path::Path;
use std::sync::Arc;

// Refuses a change to an instance while the game runs from it.
pub struct InstanceGuard<P: ProcessInspector> {
    processes: Arc<P>,
}

impl<P: ProcessInspector> Clone for InstanceGuard<P> {
    fn clone(&self) -> Self {
        InstanceGuard { processes: Arc::clone(&self.processes) }
    }
}

impl<P: ProcessInspector> InstanceGuard<P> {
    const EXE_NAME: &'static str = "Minecraft.Windows.exe";

    pub fn new(processes: Arc<P>) -> InstanceGuard<P> {
        InstanceGuard { processes }
    }

    // `InstanceRunning` when a process's image path is `<instance>\Minecraft.Windows.exe`, whatever the case or slashes
    pub fn ensure_stopped(&self, instance: &Path, name: &str) -> Result<(), LauncherError> {
        let mut wanted = vec![Self::normal(&instance.join(Self::EXE_NAME))];
        if let Ok(canonical) = std::fs::canonicalize(instance) {
            wanted.push(Self::normal(&canonical.join(Self::EXE_NAME)));
        }
        for (_, image) in self.processes.image_paths(Self::EXE_NAME)? {
            if !image.as_os_str().is_empty() && wanted.contains(&Self::normal(&image)) {
                return Err(LauncherError::InstanceRunning(name.to_string()));
            }
        }
        Ok(())
    }

    fn normal(path: &Path) -> String {
        let text = path.to_string_lossy().replace('/', "\\").to_lowercase();
        let text = text.strip_prefix("\\\\?\\").unwrap_or(&text);
        text.trim_end_matches('\\').to_string()
    }
}
