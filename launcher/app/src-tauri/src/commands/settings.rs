use crate::app::{Blocking, Session};
use amml_launcher_bridge::{AboutView, DataOverviewView, DownloadsView, FaultView, NameProblemView, SettingsView};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn settings_get(session: State<'_, Arc<Session>>) -> Result<SettingsView, FaultView> {
    Blocking::run(&session, |s| s.settings()).await
}

#[tauri::command]
pub async fn settings_set_preview(session: State<'_, Arc<Session>>, on: bool) -> Result<(), FaultView> {
    Blocking::run(&session, move |s| s.set_preview(on)).await
}

#[tauri::command]
pub async fn downloads_get(session: State<'_, Arc<Session>>) -> Result<DownloadsView, FaultView> {
    Blocking::run(&session, |s| s.downloads()).await
}

#[tauri::command]
pub async fn download_remove(session: State<'_, Arc<Session>>, id: String) -> Result<(), FaultView> {
    Blocking::run(&session, move |s| s.download_remove(&id)).await
}

#[tauri::command]
pub async fn downloads_remove_unfinished(session: State<'_, Arc<Session>>) -> Result<u32, FaultView> {
    Blocking::run(&session, |s| s.downloads_remove_unfinished()).await
}

#[tauri::command]
pub async fn data_overview(session: State<'_, Arc<Session>>) -> Result<DataOverviewView, FaultView> {
    Blocking::run(&session, |s| s.data_overview()).await
}

#[tauri::command]
pub async fn about_get(session: State<'_, Arc<Session>>) -> Result<AboutView, FaultView> {
    Blocking::run(&session, |s| s.about(env!("CARGO_PKG_VERSION"), env!("AMML_LAUNCHER_BUILT_ON"))).await
}

#[tauri::command]
pub async fn name_problem(
    session: State<'_, Arc<Session>>,
    name: String,
) -> Result<Option<NameProblemView>, FaultView> {
    Blocking::run(&session, move |s| Ok(s.name_problem(&name))).await
}

#[tauri::command]
pub async fn startup_fault(session: State<'_, Arc<Session>>) -> Result<Option<FaultView>, FaultView> {
    Blocking::run(&session, |s| Ok(s.startup_fault())).await
}
