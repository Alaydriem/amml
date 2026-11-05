use crate::{
    AmmlLevel, Compatibility, HttpFetch, InstalledMod, InstanceApi, InstanceState, LauncherError, ModCatalogue,
    ModSource, NeedsAmml, UpdateRow, UpdateStatus,
};
use amml_registry::{RegistryRoot, ReleaseVersion};

// Compares what an instance holds with what the registries offer. It reports and installs nothing.
pub struct UpdateChecker;

impl UpdateChecker {
    pub const AMML_ID: &'static str = "alaydriem/amml";

    pub fn check<H: HttpFetch>(catalogue: &mut ModCatalogue<H>, state: &InstanceState) -> Vec<UpdateRow> {
        catalogue.registries_mut().refresh_all();
        Self::check_fetched(catalogue, state)
    }

    // Like `check`, but uses what the registries already hold and fetches no registry root again.
    pub fn check_fetched<H: HttpFetch>(catalogue: &mut ModCatalogue<H>, state: &InstanceState) -> Vec<UpdateRow> {
        let instance_api = Self::instance_api(catalogue, state);
        let level = AmmlLevel::of(&state.amml);
        let root = catalogue.registries_mut().official_root().cloned();
        let mut rows = vec![Self::amml_row(catalogue, state)];
        for installed in &state.mods {
            rows.push(Self::mod_row(catalogue, installed, instance_api, &level, root.as_ref()));
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
            needs_amml: None,
        };
        let root = catalogue.registries_mut().official_mut().and_then(|client| client.root().cloned());
        // a local build is never ordered against a release; the newest release for the game is only offered
        if state.amml.local {
            let release = root.as_ref().and_then(|root| Compatibility::amml_for(&state.game, root));
            let api_change =
                release.as_ref().zip(state.amml.local_api).is_some_and(|(release, api)| release.api_version != api);
            return row(release.map(|release| release.version), UpdateStatus::LocalBuild, api_change);
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
        level: &AmmlLevel,
        root: Option<&RegistryRoot>,
    ) -> UpdateRow {
        let row = |available: Option<String>, status: UpdateStatus, needs_amml: Option<NeedsAmml>| UpdateRow {
            id: installed.id.as_str(),
            installed: installed.version.clone(),
            available,
            status,
            api_change: false,
            needs_amml,
        };
        if matches!(installed.source, ModSource::Url(_)) {
            return row(None, UpdateStatus::Unofficial, None);
        }
        if installed.version.is_none() || installed.api_version == 0 {
            return row(None, UpdateStatus::Unknown, None);
        }
        // the mod's own API stands in only while the AMML version is not known
        let api = instance_api.unwrap_or(installed.api_version);
        let (available, status, needs_amml) = match &installed.source {
            ModSource::Registry(base) => Self::against_registry(catalogue, installed, api, level, root, Some(base)),
            _ => Self::against_registry(catalogue, installed, api, level, root, None),
        };
        row(available, status, needs_amml)
    }

    // The registry named by `only`, or every registry in settings order when `only` is `None`.
    fn against_registry<H: HttpFetch>(
        catalogue: &mut ModCatalogue<H>,
        installed: &InstalledMod,
        api: u32,
        level: &AmmlLevel,
        root: Option<&RegistryRoot>,
        only: Option<&String>,
    ) -> (Option<String>, UpdateStatus, Option<NeedsAmml>) {
        let mut offline = false;
        for client in catalogue.registries_mut().clients_mut() {
            if only.is_some_and(|base| &client.registry().base != base) {
                continue;
            }
            match client.document(&installed.id) {
                Ok(cached) => {
                    let mut choice = Compatibility::choose(&cached.value, api, level, root);
                    if Self::has_newest(installed, &cached.value, api) {
                        choice.blocked = None;
                    }
                    let Some(fitting) = choice.fitting else {
                        if choice.blocked.is_some() {
                            return (None, UpdateStatus::NeedsAmml, choice.blocked);
                        }
                        // a mod built for another API than the instance's has nothing that fits it
                        let status =
                            if installed.api_version != api { UpdateStatus::Unknown } else { UpdateStatus::UpToDate };
                        return (None, status, None);
                    };
                    return match Self::compare(installed.version.as_deref(), &fitting.version) {
                        // the fitting release is not newer, but a blocked one is
                        UpdateStatus::UpToDate if choice.blocked.is_some() => {
                            (None, UpdateStatus::NeedsAmml, choice.blocked)
                        }
                        // `Unknown` here: the installed version cannot be ordered, so the blocked note is kept
                        status => (Some(fitting.version), status, choice.blocked),
                    };
                }
                Err(LauncherError::Offline { .. }) => offline = true,
                Err(_) => {}
            }
        }
        (None, if offline { UpdateStatus::Offline } else { UpdateStatus::NotListed }, None)
    }

    // true when the installed release is equal to or newer than every release for the API, even those that do not fit
    fn has_newest(installed: &InstalledMod, document: &amml_registry::ModDocument, api: u32) -> bool {
        let newest = Compatibility::newest_for_api(document, api)
            .and_then(|release| ReleaseVersion::parse(&release.version).ok());
        let have = installed.version.as_deref().and_then(|text| ReleaseVersion::parse(text).ok());
        newest.zip(have).is_some_and(|(newest, have)| have >= newest)
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
