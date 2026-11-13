use crate::app::{LauncherUpdater, Session};
use amml_launcher_bridge::{FaultView, JobStateView, LauncherUpdateView};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn update_check(updater: State<'_, LauncherUpdater>) -> Result<Option<LauncherUpdateView>, String> {
    Ok(updater.check().await)
}

#[tauri::command]
pub async fn update_install(
    updater: State<'_, LauncherUpdater>,
    session: State<'_, Arc<Session>>,
) -> Result<(), FaultView> {
    let job_running = session.job().is_some_and(|job| job.state == JobStateView::Running);
    updater.install(job_running).await
}
