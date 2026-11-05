use crate::{
    AdoptionCandidate, DataChoice, InstanceHclFile, InstanceReader, InstanceState, LauncherBlock, LauncherError,
    LauncherModEntry, LauncherPaths, ModSource, VersionJsonFile,
};
use amml_gdk::{InstanceService, VersionStore};
use std::collections::BTreeMap;

// Brings an instance made by another launcher under this launcher's care without changing its files, except
// `version.json` and a new `amml\instance.hcl`.
pub struct Adopter;

impl Adopter {
    pub fn candidates(paths: &LauncherPaths) -> Vec<AdoptionCandidate> {
        let store = VersionStore::new(paths.gdk().clone());
        InstanceReader::list(paths)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|state| !state.managed)
            .map(|state| AdoptionCandidate {
                has_version_in_store: store.get(&state.game).is_some(),
                name: state.name,
                game: state.game,
                amml: state.amml.version,
                amml_local: state.amml.local,
                mod_count: state.mods.len(),
            })
            .collect()
    }

    // An instance the launcher already manages is returned as it is.
    pub fn adopt(
        paths: &LauncherPaths,
        instances: &InstanceService,
        store: &VersionStore,
        name: &str,
        data: &DataChoice,
    ) -> Result<InstanceState, LauncherError> {
        let folder = paths.checked_instance(name)?;
        let state = InstanceReader::read(&folder)?;
        if state.managed {
            return Ok(state);
        }

        let version_path = folder.join(InstanceReader::VERSION_JSON);
        let mut version_json = VersionJsonFile::read(&version_path)?;
        version_json.set_isolation(false);
        let mods: BTreeMap<String, LauncherModEntry> = state
            .mods
            .iter()
            .map(|installed| {
                (
                    installed.id.as_str(),
                    LauncherModEntry { version: installed.version.clone(), source: ModSource::Local, core: None },
                )
            })
            .collect();
        version_json.set_launcher(&LauncherBlock::new(state.amml.version.clone(), mods));
        version_json.write(&version_path)?;

        let hcl = InstanceReader::instance_hcl(&folder);
        if !InstanceHclFile::exists(&hcl) {
            InstanceHclFile::write(&hcl, data)?;
        }
        if store.get(&state.game).is_none()
            && let Err(error) = instances.seed_version_from_instance(&folder, &state.game)
        {
            curia::warn!("the game version was not copied into the version store", {
                instance: name,
                error: error.to_string(),
            });
        }
        InstanceReader::read(&folder)
    }
}
