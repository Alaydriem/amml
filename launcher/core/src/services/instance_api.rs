use crate::InstanceState;
use amml_registry::RegistryRoot;

// The AMML API an instance runs.
pub struct InstanceApi;

impl InstanceApi {
    // A core release: the official registry's entry for its version. A local build: its `amml-api.txt`. Else the
    // first mod that names its API.
    pub fn of(state: &InstanceState, root: Option<&RegistryRoot>) -> Option<u32> {
        if state.amml.local {
            if let Some(api) = state.amml.local_api {
                return Some(api);
            }
        } else if let (Some(version), Some(root)) = (&state.amml.version, root)
            && let Some(release) = root.amml.versions.iter().find(|release| &release.version == version)
        {
            return Some(release.api_version);
        }
        state.mods.iter().map(|installed| installed.api_version).find(|api| *api != 0)
    }
}
