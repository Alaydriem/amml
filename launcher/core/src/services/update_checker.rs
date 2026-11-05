use crate::{
    Compatibility, HttpFetch, InstalledMod, InstanceApi, InstanceState, LauncherError, ModCatalogue, ModSource,
    UpdateRow, UpdateStatus,
};
use amml_registry::ReleaseVersion;

// Compares what an instance holds with what the registries offer. It reports and installs nothing.
pub struct UpdateChecker;

impl UpdateChecker {
    pub const AMML_ID: &'static str = "alaydriem/amml";

    pub fn check<H: HttpFetch>(catalogue: &mut ModCatalogue<H>, state: &InstanceState) -> Vec<UpdateRow> {
        catalogue.registries_mut().refresh_all();
        let instance_api = Self::instance_api(catalogue, state);
        let mut rows = vec![Self::amml_row(catalogue, state)];
        for installed in &state.mods {
            rows.push(Self::mod_row(catalogue, installed, instance_api));
        }
        rows
    }

    // the API of the AMML in the instance, from the official registry's entry for its version
    fn instance_api<H: HttpFetch>(catalogue: &mut ModCatalogue<H>, state: &InstanceState) -> Option<u32> {
        let root = catalogue.registries_mut().official_mut().and_then(|client| client.root().cloned());
        InstanceApi::of(state, root.as_ref())
    }

    fn amml_row<H: HttpFetch>(catalogue: &mut ModCatalogue<H>, state: &InstanceState) -> UpdateRow {
        let installed = state.amml.version.clone();
        let row = |available: Option<String>, status: UpdateStatus, api_change: bool| UpdateRow {
            id: Self::AMML_ID.to_string(),
            installed: installed.clone(),
            available,
            status,
            api_change,
        };
        let root = catalogue.registries_mut().official_mut().and_then(|client| client.root().cloned());
        // a local build is never ordered against a release; the newest release for the game is only offered
        if state.amml.local {
            let release = root.as_ref().and_then(|root| Compatibility::amml_for(&state.game, root));
            return row(release.map(|release| release.version), UpdateStatus::LocalBuild, false);
        }
        let Some(root) = root else {
            return row(None, UpdateStatus::Offline, false);
        };
        let Some(release) = Compatibility::amml_for(&state.game, &root) else {
            return row(None, UpdateStatus::NotListed, false);
        };
        let installed_api = installed
            .as_ref()
            .and_then(|version| root.amml.versions.iter().find(|listed| &listed.version == version))
            .map(|listed| listed.api_version);
        let api_change = installed_api.is_some_and(|api| api != release.api_version);
        let status = Self::compare(installed.as_deref(), &release.version);
        row(Some(release.version), status, api_change)
    }

    fn mod_row<H: HttpFetch>(
        catalogue: &mut ModCatalogue<H>,
        installed: &InstalledMod,
        instance_api: Option<u32>,
    ) -> UpdateRow {
        let row = |available: Option<String>, status: UpdateStatus| UpdateRow {
            id: installed.id.as_str(),
            installed: installed.version.clone(),
            available,
            status,
            api_change: false,
        };
        if matches!(installed.source, ModSource::Url(_)) {
            return row(None, UpdateStatus::Unofficial);
        }
        if installed.version.is_none() || installed.api_version == 0 {
            return row(None, UpdateStatus::Unknown);
        }
        // the mod's own API stands in only while the AMML version is not known
        let api = instance_api.unwrap_or(installed.api_version);
        let (available, status) = match &installed.source {
            ModSource::Registry(base) => Self::against_registry(catalogue, installed, api, Some(base)),
            _ => Self::against_registry(catalogue, installed, api, None),
        };
        row(available, status)
    }

    // The registry named by `only`, or every registry in settings order when `only` is `None`.
    fn against_registry<H: HttpFetch>(
        catalogue: &mut ModCatalogue<H>,
        installed: &InstalledMod,
        api: u32,
        only: Option<&String>,
    ) -> (Option<String>, UpdateStatus) {
        let mut offline = false;
        for client in catalogue.registries_mut().clients_mut() {
            if only.is_some_and(|base| &client.registry().base != base) {
                continue;
            }
            match client.document(&installed.id) {
                Ok(cached) => {
                    return match Compatibility::newest_for_api(&cached.value, api) {
                        Some(release) => {
                            let status = Self::compare(installed.version.as_deref(), &release.version);
                            (Some(release.version), status)
                        }
                        // a mod built for another API than the instance's has nothing that fits it
                        None if installed.api_version != api => (None, UpdateStatus::Unknown),
                        None => (None, UpdateStatus::UpToDate),
                    };
                }
                Err(LauncherError::Offline { .. }) => offline = true,
                Err(_) => {}
            }
        }
        (None, if offline { UpdateStatus::Offline } else { UpdateStatus::NotListed })
    }

    // `Available` when the offered version is newer by number; `Unknown` when the installed version is not a version
    fn compare(installed: Option<&str>, offered: &str) -> UpdateStatus {
        let (Some(installed), Ok(offered)) = (installed, ReleaseVersion::parse(offered)) else {
            return UpdateStatus::Unknown;
        };
        match ReleaseVersion::parse(installed) {
            Ok(have) if have >= offered => UpdateStatus::UpToDate,
            Ok(_) => UpdateStatus::Available,
            Err(_) => UpdateStatus::Unknown,
        }
    }
}
