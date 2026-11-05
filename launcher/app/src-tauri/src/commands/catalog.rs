use crate::app::{Blocking, Session};
use amml_launcher_bridge::{
    CardView, CatalogLayoutView, CatalogSortView, ChannelView, FaultView, ModDetailView, ModView, RegistryView,
    SearchGroupView, UrlCheckView, VersionView,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn mods_list(session: State<'_, Arc<Session>>, name: String) -> Result<Vec<ModView>, FaultView> {
    Blocking::run(&session, move |s| s.mods(&name)).await
}

#[tauri::command]
pub async fn mod_set_enabled(
    session: State<'_, Arc<Session>>,
    name: String,
    id: String,
    enabled: bool,
) -> Result<(), FaultView> {
    Blocking::run(&session, move |s| s.set_mod_enabled(&name, &id, enabled)).await
}

#[tauri::command]
pub async fn mod_remove(session: State<'_, Arc<Session>>, name: String, id: String) -> Result<(), FaultView> {
    Blocking::run(&session, move |s| s.remove_mod(&name, &id)).await
}

#[tauri::command]
pub async fn url_inspect(
    session: State<'_, Arc<Session>>,
    name: String,
    url: String,
) -> Result<UrlCheckView, FaultView> {
    Blocking::run(&session, move |s| s.inspect_url(&name, &url)).await
}

#[tauri::command]
pub async fn catalog_layout(
    session: State<'_, Arc<Session>>,
    name: String,
    registry_id: String,
) -> Result<CatalogLayoutView, FaultView> {
    Blocking::run(&session, move |s| s.catalog_layout(&name, &registry_id)).await
}

#[tauri::command]
pub async fn catalog_page(
    session: State<'_, Arc<Session>>,
    name: String,
    registry_id: String,
    page: u32,
) -> Result<Vec<CardView>, FaultView> {
    Blocking::run(&session, move |s| s.catalog_page(&name, &registry_id, page)).await
}

#[tauri::command]
pub async fn catalog_search(
    session: State<'_, Arc<Session>>,
    name: String,
    query: String,
) -> Result<Vec<SearchGroupView>, FaultView> {
    Blocking::run(&session, move |s| s.catalog_search(&name, &query)).await
}

#[tauri::command]
pub async fn catalog_sorted(
    session: State<'_, Arc<Session>>,
    name: String,
    registry_id: String,
    sort: CatalogSortView,
) -> Result<Option<Vec<CardView>>, FaultView> {
    Blocking::run(&session, move |s| s.catalog_sorted(&name, &registry_id, sort)).await
}

#[tauri::command]
pub async fn catalog_detail(
    session: State<'_, Arc<Session>>,
    name: String,
    registry_id: String,
    id: String,
) -> Result<ModDetailView, FaultView> {
    Blocking::run(&session, move |s| s.catalog_detail(&name, &registry_id, &id)).await
}

#[tauri::command]
pub async fn registries_refresh(session: State<'_, Arc<Session>>) -> Result<Vec<RegistryView>, FaultView> {
    Blocking::run(&session, |s| s.refresh_registries()).await
}

#[tauri::command]
pub async fn registries_get(session: State<'_, Arc<Session>>) -> Result<Vec<RegistryView>, FaultView> {
    Blocking::run(&session, |s| Ok(s.registry_views())).await
}

#[tauri::command]
pub async fn registry_add(session: State<'_, Arc<Session>>, url: String) -> Result<(), FaultView> {
    Blocking::run(&session, move |s| s.add_registry(&url)).await
}

#[tauri::command]
pub async fn registry_remove(session: State<'_, Arc<Session>>, base: String) -> Result<(), FaultView> {
    Blocking::run(&session, move |s| s.remove_registry(&base)).await
}

#[tauri::command]
pub async fn versions_list(
    session: State<'_, Arc<Session>>,
    channel: ChannelView,
) -> Result<Vec<VersionView>, FaultView> {
    Blocking::run(&session, move |s| s.version_list(channel)).await
}

#[tauri::command]
pub async fn versions_on_disk(session: State<'_, Arc<Session>>) -> Result<Vec<VersionView>, FaultView> {
    Blocking::run(&session, |s| s.versions_on_disk()).await
}

#[tauri::command]
pub async fn version_remove(session: State<'_, Arc<Session>>, version: String) -> Result<bool, FaultView> {
    Blocking::run(&session, move |s| s.remove_version(&version)).await
}
