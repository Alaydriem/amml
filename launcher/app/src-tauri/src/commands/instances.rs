use crate::app::{Blocking, Session};
use amml_launcher_bridge::{FaultView, InstanceView};
use std::sync::Arc;
use tauri::State;

// Tauri commands are free functions by framework rule. Each one only delegates to one bridge call.

#[tauri::command]
pub async fn instances_list(session: State<'_, Arc<Session>>) -> Result<Vec<InstanceView>, FaultView> {
    Blocking::run(&session, |s| s.instances()).await
}

#[tauri::command]
pub async fn instance_get(session: State<'_, Arc<Session>>, name: String) -> Result<InstanceView, FaultView> {
    Blocking::run(&session, move |s| s.instance(&name)).await
}

#[tauri::command]
pub async fn instance_create(
    session: State<'_, Arc<Session>>,
    name: String,
    game: String,
    preview: bool,
    own_data: bool,
) -> Result<u64, FaultView> {
    Blocking::run(&session, move |s| s.create(&name, &game, preview, own_data)).await
}

#[tauri::command]
pub async fn instance_delete(session: State<'_, Arc<Session>>, name: String, confirm: String) -> Result<(), FaultView> {
    Blocking::run(&session, move |s| s.delete(&name, &confirm)).await
}

#[tauri::command]
pub async fn instance_launch(session: State<'_, Arc<Session>>, name: String) -> Result<(), FaultView> {
    Blocking::run(&session, move |s| s.launch(&name)).await
}
