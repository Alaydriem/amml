use amml_launcher_core::InstanceState;
use amml_registry::RegistryRoot;

// The AMML API an instance runs.
pub struct InstanceApi;

impl InstanceApi {
    // From the official registry's entry for the installed AMML version, else from a mod that names its API.
    pub fn of(state: &InstanceState, root: Option<&RegistryRoot>) -> Option<u32> {
        if let (Some(version), Some(root)) = (&state.amml.version, root)
            && let Some(release) = root.amml.versions.iter().find(|release| &release.version == version)
        {
            return Some(release.api_version);
        }
        state.mods.iter().map(|installed| installed.api_version).find(|api| *api != 0)
    }
}
