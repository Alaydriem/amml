// What an instance holds of AMML: `installed` when `amml.dll` is there, `version` from `amml\amml-version.txt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmmlState {
    pub version: Option<String>,
    pub installed: bool,
}
