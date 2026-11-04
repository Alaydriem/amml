use crate::app::{Blocking, Session};
use amml_launcher_bridge::{FaultView, JobSnapshot};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn mods_install(session: State<'_, Arc<Session>>, name: String, ids: Vec<String>) -> Result<u64, FaultView> {
    Blocking::run(&session, move |s| s.install_mods(&name, ids)).await
}

#[tauri::command]
pub async fn mods_update(session: State<'_, Arc<Session>>, name: String, ids: Vec<String>) -> Result<u64, FaultView> {
    Blocking::run(&session, move |s| s.update_mods(&name, ids)).await
}

#[tauri::command]
pub async fn amml_update(session: State<'_, Arc<Session>>, name: String) -> Result<u64, FaultView> {
    Blocking::run(&session, move |s| s.update_amml(&name)).await
}

#[tauri::command]
pub async fn url_install(
    session: State<'_, Arc<Session>>,
    name: String,
    url: String,
    consent: bool,
) -> Result<u64, FaultView> {
    Blocking::run(&session, move |s| s.install_url(&name, &url, consent)).await
}

#[tauri::command]
pub async fn worlds_copy(session: State<'_, Arc<Session>>, source: String) -> Result<u64, FaultView> {
    Blocking::run(&session, move |s| s.copy_worlds(&source)).await
}

#[tauri::command]
pub async fn game_input_install(session: State<'_, Arc<Session>>) -> Result<u64, FaultView> {
    Blocking::run(&session, |s| s.install_game_input()).await
}

#[tauri::command]
pub async fn setup_run(session: State<'_, Arc<Session>>, source: String) -> Result<u64, FaultView> {
    Blocking::run(&session, move |s| s.set_up_data(&source)).await
}

#[tauri::command]
pub async fn job_get(session: State<'_, Arc<Session>>) -> Result<Option<JobSnapshot>, FaultView> {
    Blocking::run(&session, |s| Ok(s.job())).await
}

#[tauri::command]
pub async fn job_cancel(session: State<'_, Arc<Session>>) -> Result<(), FaultView> {
    Blocking::run(&session, |s| {
        s.cancel_job();
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn job_retry(session: State<'_, Arc<Session>>) -> Result<u64, FaultView> {
    Blocking::run(&session, |s| s.retry()).await
}

#[tauri::command]
pub async fn job_retry_developer_mode(session: State<'_, Arc<Session>>) -> Result<u64, FaultView> {
    Blocking::run(&session, |s| s.retry_with_developer_mode()).await
}
