use crate::app::{AppOpener, Blocking, Session};
use amml_launcher_bridge::{AdoptCandidatesView, CheckView, FaultView, SetupView};
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn checks_run(session: State<'_, Arc<Session>>) -> Result<Vec<CheckView>, FaultView> {
    Blocking::run(&session, |s| s.checks()).await
}

#[tauri::command]
pub async fn setup_get(session: State<'_, Arc<Session>>) -> Result<SetupView, FaultView> {
    Blocking::run(&session, |s| Ok(s.setup())).await
}

#[tauri::command]
pub async fn adopt_candidates(session: State<'_, Arc<Session>>) -> Result<AdoptCandidatesView, FaultView> {
    Blocking::run(&session, |s| s.adopt_candidates()).await
}

#[tauri::command]
pub async fn open_path(app: AppHandle, session: State<'_, Arc<Session>>, path: String) -> Result<(), FaultView> {
    AppOpener::path(&app, &session, path).await
}

#[tauri::command]
pub async fn open_url(app: AppHandle, session: State<'_, Arc<Session>>, url: String) -> Result<(), FaultView> {
    AppOpener::url(&app, &session, url).await
}
