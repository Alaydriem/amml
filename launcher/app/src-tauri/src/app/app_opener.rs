use super::{Blocking, Session};
use amml_launcher_bridge::FaultView;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

// Opens a folder or a web page with the system, after the bridge says the window may.
pub struct AppOpener;

impl AppOpener {
    pub async fn path(app: &AppHandle, session: &State<'_, Arc<Session>>, path: String) -> Result<(), FaultView> {
        let checked = Blocking::run(session, move |s| s.check_open_path(&path)).await?;
        app.opener()
            .open_path(checked.display().to_string(), None::<&str>)
            .map_err(|error| Blocking::fault(error.to_string()))
    }

    pub async fn url(app: &AppHandle, session: &State<'_, Arc<Session>>, url: String) -> Result<(), FaultView> {
        let checked = Blocking::run(session, move |s| s.check_open_url(&url)).await?;
        app.opener().open_url(checked, None::<&str>).map_err(|error| Blocking::fault(error.to_string()))
    }
}
