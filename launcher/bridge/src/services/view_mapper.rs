use super::UnixMillis;
use crate::{
    DataModeView, InstanceView, ModOriginView, ModView, NeedsAmmlView, RegistryStatusView, RegistryView,
    UpdateStatusView,
};
use amml_launcher_core::{
    InstalledMod, InstanceState, LauncherPaths, LauncherSettings, ModSource, RegistryRef, RegistryStatus, RunningGame,
    UpdateChecker, UpdateRow, UpdateStatus,
};
use amml_registry::RegistryRoot;
use std::time::SystemTime;

// Turns core types into the facts the UI reads. It words nothing.
pub struct ViewMapper;

impl ViewMapper {
    pub fn instance(
        state: &InstanceState,
        settings: &LauncherSettings,
        paths: &LauncherPaths,
        running: &[RunningGame],
        updates: u32,
        last_played: Option<SystemTime>,
    ) -> InstanceView {
        let own = state.data_dir.as_ref().is_some_and(|dir| dir.starts_with(&state.folder));
        let data_dir = state.data_dir.clone().unwrap_or_else(|| settings.data_dir(paths));
        InstanceView {
            name: state.name.clone(),
            folder: state.folder.display().to_string(),
            game: state.game.instance_name(),
            preview: state.preview,
            amml_version: state.amml.version.clone(),
            amml_local: state.amml.local,
            amml_installed: state.amml.installed,
            mod_count: state.mods.len() as u32,
            data_mode: if own { DataModeView::Own } else { DataModeView::Shared },
            data_dir: Some(data_dir.display().to_string()),
            managed: state.managed,
            running_since: running
                .iter()
                .find(|game| game.instance == state.name)
                .map(|game| UnixMillis::of(game.started)),
            last_played: last_played.map(UnixMillis::of),
            update_count: updates,
        }
    }

    // the AMML row first, then each mod of the instance in the order the core read them
    pub fn mods(state: &InstanceState, rows: &[UpdateRow], core_ids: &[String]) -> Vec<ModView> {
        let mut views = Vec::with_capacity(state.mods.len() + 1);
        let amml_row = rows.iter().find(|row| row.id == UpdateChecker::AMML_ID);
        views.push(ModView {
            id: UpdateChecker::AMML_ID.to_string(),
            title: None,
            origin: ModOriginView::Core,
            registry_base: None,
            enabled: true,
            installed_version: state.amml.version.clone(),
            available_version: amml_row.and_then(|row| row.available.clone()),
            api: 0,
            update: amml_row.map_or(UpdateStatusView::Unknown, |row| Self::update_status(row.status)),
            needs_amml: None,
        });
        for installed in &state.mods {
            let id = installed.id.as_str();
            let row = rows.iter().find(|row| row.id == id);
            let update = Self::mod_update(installed, row);
            let needs_amml = match update {
                UpdateStatusView::NeedsAmml | UpdateStatusView::Available => {
                    row.and_then(|row| row.needs_amml.as_ref()).map(NeedsAmmlView::from)
                }
                _ => None,
            };
            views.push(ModView {
                origin: Self::origin(installed, core_ids.contains(&id)),
                registry_base: match &installed.source {
                    ModSource::Registry(base) => Some(base.clone()),
                    _ => None,
                },
                title: installed.listing.as_ref().map(|listing| listing.title.clone()),
                enabled: installed.enabled,
                installed_version: installed.version.clone(),
                available_version: row.and_then(|row| row.available.clone()),
                api: installed.api_version,
                update,
                needs_amml,
                id,
            });
        }
        views
    }

    pub fn registry(
        client_ref: &RegistryRef,
        root: Option<&RegistryRoot>,
        status: &RegistryStatus,
        api: Option<u32>,
        fetched: Option<SystemTime>,
    ) -> RegistryView {
        let mod_count = root.map(|root| match api {
            Some(api) => root.index.apis.get(&api.to_string()).map_or(0, |summary| summary.mods),
            None => root.mod_count,
        });
        RegistryView {
            id: client_ref.cache_key(),
            base: client_ref.base.clone(),
            name: root.map(|root| root.name.clone()),
            official: client_ref.is_official(),
            status: match status {
                RegistryStatus::Ok => RegistryStatusView::Ok,
                RegistryStatus::Stale { .. } => RegistryStatusView::Stale,
                RegistryStatus::Offline => RegistryStatusView::Offline,
                RegistryStatus::Broken { .. } => RegistryStatusView::Broken,
            },
            mod_count,
            fetched: fetched.map(UnixMillis::of),
        }
    }

    pub fn update_status(status: UpdateStatus) -> UpdateStatusView {
        match status {
            UpdateStatus::UpToDate => UpdateStatusView::UpToDate,
            UpdateStatus::Available => UpdateStatusView::Available,
            UpdateStatus::Unknown => UpdateStatusView::Unknown,
            UpdateStatus::NotListed => UpdateStatusView::NotListed,
            UpdateStatus::Unofficial => UpdateStatusView::Unofficial,
            UpdateStatus::Offline => UpdateStatusView::Offline,
            UpdateStatus::LocalBuild => UpdateStatusView::LocalBuild,
            UpdateStatus::NeedsAmml => UpdateStatusView::NeedsAmml,
        }
    }

    fn origin(installed: &InstalledMod, core: bool) -> ModOriginView {
        if core {
            return ModOriginView::Core;
        }
        match installed.source {
            ModSource::Registry(_) => ModOriginView::Registry,
            ModSource::Url(_) => ModOriginView::Url,
            ModSource::Local => ModOriginView::Local,
        }
    }

    // a mod without a version is never "up to date"; a URL mod is never checked
    fn mod_update(installed: &InstalledMod, row: Option<&UpdateRow>) -> UpdateStatusView {
        if matches!(installed.source, ModSource::Url(_)) {
            return UpdateStatusView::Unofficial;
        }
        match row {
            Some(row) if installed.version.is_some() => Self::update_status(row.status),
            _ => UpdateStatusView::Unknown,
        }
    }
}
