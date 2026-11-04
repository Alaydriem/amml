use super::{LanguagePacks, LogSetup, TauriJobSink, WindowFocus};
use crate::commands;
use amml_gdk::{AmmlPaths, CancelFlag};
use amml_launcher_bridge::LauncherSession;
use amml_launcher_core::{CoreServices, LauncherPaths};
use std::sync::Arc;
use tauri::path::BaseDirectory;
use tauri::{Manager, Wry};

// The session every command reaches through Tauri's managed state.
pub type Session = LauncherSession<TauriJobSink>;

// Builds and runs the launcher window.
pub struct LauncherApp;

impl LauncherApp {
    pub fn run() {
        let context = tauri::generate_context!();
        if let Err(error) = Self::builder().run(context) {
            curia::error!("the launcher stopped", { error: error.to_string() });
        }
    }

    fn builder() -> tauri::Builder<Wry> {
        tauri::Builder::default()
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| WindowFocus::main(app)))
            .plugin(tauri_plugin_curia::init())
            .plugin(tauri_plugin_opener::init())
            .setup(|app| Self::setup(app))
            .invoke_handler(tauri::generate_handler![
                commands::i18n::i18n_locales,
                commands::i18n::i18n_load,
                commands::instances::instances_list,
                commands::instances::instance_get,
                commands::instances::instance_create,
                commands::instances::instance_delete,
                commands::instances::instance_launch,
                commands::catalog::mods_list,
                commands::catalog::mod_set_enabled,
                commands::catalog::mod_remove,
                commands::jobs::mods_install,
                commands::jobs::mods_update,
                commands::jobs::amml_update,
                commands::catalog::url_inspect,
                commands::jobs::url_install,
                commands::catalog::catalog_layout,
                commands::catalog::catalog_page,
                commands::catalog::catalog_search,
                commands::catalog::catalog_sorted,
                commands::catalog::catalog_detail,
                commands::catalog::registries_refresh,
                commands::catalog::registry_add,
                commands::catalog::registry_remove,
                commands::catalog::versions_list,
                commands::catalog::versions_on_disk,
                commands::catalog::version_remove,
                commands::system::checks_run,
                commands::jobs::game_input_install,
                commands::system::setup_get,
                commands::system::adopt_candidates,
                commands::jobs::setup_run,
                commands::settings::data_overview,
                commands::jobs::worlds_copy,
                commands::settings::downloads_get,
                commands::settings::download_remove,
                commands::settings::downloads_remove_unfinished,
                commands::settings::settings_get,
                commands::settings::settings_set_preview,
                commands::jobs::job_get,
                commands::jobs::job_cancel,
                commands::jobs::job_retry,
                commands::jobs::job_retry_developer_mode,
                commands::settings::about_get,
                commands::settings::name_problem,
                commands::settings::startup_fault,
                commands::system::open_path,
                commands::system::open_url,
            ])
    }

    fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
        let root = AmmlPaths::default_root()?;
        LogSetup::install(root.join("launcher").join("logs"))?;
        curia::info!("the launcher starts", { version: env!("CARGO_PKG_VERSION") });

        let packs = app.path().resolve("resources/i18n", BaseDirectory::Resource)?;
        app.manage(LanguagePacks::new(packs));

        let cancel = CancelFlag::new();
        let services = CoreServices::new(LauncherPaths::new(AmmlPaths::new(root)), None, cancel.clone());
        let session: Arc<Session> =
            Arc::new(LauncherSession::new(services, TauriJobSink::new(app.handle().clone()), cancel));
        app.manage(Arc::clone(&session));

        // finishing a cut-off install can take a while, so it runs off the window thread
        std::thread::Builder::new().name("launcher-recover".to_string()).spawn(move || {
            session.recover_install();
        })?;
        Ok(())
    }
}
