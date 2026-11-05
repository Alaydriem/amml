// What an instance holds of AMML: `installed` when `amml.dll` is there, `version` from `amml\amml-version.txt`.
// `local` when that version ends in `+local` (a build that `install-game` put in, not a core release);
// `local_api` from `amml\amml-api.txt`, read only for a local build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmmlState {
    pub version: Option<String>,
    pub installed: bool,
    pub local: bool,
    pub local_api: Option<u32>,
}

impl AmmlState {
    pub const LOCAL_SUFFIX: &'static str = "+local";

    // the version as people read it: `26.60.5+local (local build)`, `26.60.1`, or `none`
    pub fn label(version: Option<&str>, local: bool) -> String {
        match (version, local) {
            (Some(version), true) => format!("{version} (local build)"),
            (Some(version), false) => version.to_string(),
            (None, _) => "none".to_string(),
        }
    }
}
