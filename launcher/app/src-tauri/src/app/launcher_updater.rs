use super::Blocking;
use amml_gdk::PackageContext;
use amml_launcher_bridge::{BundleKind, FaultKindView, FaultView, InstallKind, LauncherUpdateView};
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::AppHandle;
use tauri::utils::config::BundleType;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::{Update, UpdaterExt};

// Looks for a newer launcher and installs it, or sends the player to the release page when the install cannot update
// itself.
pub struct LauncherUpdater {
    app: AppHandle,
    kind: InstallKind,
    found: Mutex<Option<Update>>,
    installing: AtomicBool,
}

impl LauncherUpdater {
    pub fn new(app: AppHandle, kind: InstallKind) -> LauncherUpdater {
        LauncherUpdater { app, kind, found: Mutex::new(None), installing: AtomicBool::new(false) }
    }

    // The install kind of the running process: package identity first, then the bundle the build came from.
    pub fn detect_kind() -> InstallKind {
        let packaged = PackageContext::new().is_packaged();
        InstallKind::detect(packaged, Self::bundle_kind(tauri::utils::platform::bundle_type()))
    }

    // Only the NSIS and MSI bundles map; any other value is a build the updater does not install over.
    pub fn bundle_kind(bundle: Option<BundleType>) -> Option<BundleKind> {
        match bundle {
            Some(BundleType::Nsis) => Some(BundleKind::Nsis),
            Some(BundleType::Msi) => Some(BundleKind::Msi),
            _ => None,
        }
    }

    // A failed check is one warning and no update, never an error for the window.
    pub async fn check(&self) -> Option<LauncherUpdateView> {
        if !self.kind.checks_github() {
            return None;
        }
        match self.look().await {
            Ok(Some(update)) => {
                let view = LauncherUpdateView::new(&update.version, self.kind.self_updates());
                *self.held() = Some(update);
                Some(view)
            }
            Ok(None) => {
                *self.held() = None;
                None
            }
            Err(error) => {
                curia::warn!("update check failed", { error: error });
                None
            }
        }
    }

    // Refuses with `Busy` while another install runs. `job_running` is the session's word on a launcher job.
    pub async fn install(&self, job_running: bool) -> Result<(), FaultView> {
        let busy = || FaultView::new(FaultKindView::Busy, None, BTreeMap::new());
        if job_running {
            return Err(busy());
        }
        if self.installing.swap(true, Ordering::SeqCst) {
            return Err(busy());
        }
        let result = self.run_install().await;
        if result.is_err() {
            self.installing.store(false, Ordering::SeqCst);
        }
        result.map_err(Blocking::fault)
    }

    async fn run_install(&self) -> Result<(), String> {
        let held = self.held().take();
        let update = match held {
            Some(update) => update,
            None => match self.look().await? {
                Some(update) => update,
                None => {
                    self.installing.store(false, Ordering::SeqCst);
                    return Ok(());
                }
            },
        };
        if !self.kind.self_updates() {
            let page = LauncherUpdateView::new(&update.version, false).page;
            let opened = self.app.opener().open_url(page, None::<&str>).map_err(|error| error.to_string());
            self.installing.store(false, Ordering::SeqCst);
            return opened;
        }
        update.download_and_install(|_, _| {}, || {}).await.map_err(|error| error.to_string())
    }

    async fn look(&self) -> Result<Option<Update>, String> {
        let updater =
            self.app.updater_builder().timeout(Duration::from_secs(30)).build().map_err(|error| error.to_string())?;
        updater.check().await.map_err(|error| error.to_string())
    }

    fn held(&self) -> std::sync::MutexGuard<'_, Option<Update>> {
        self.found.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
