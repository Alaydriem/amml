use super::VersionFolder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    pub version: VersionFolder,
    // `"copy-out"` or `"install-restore"`
    pub path_used: &'static str,
    // the Bedrock package registered when the install ended
    pub final_full_name: String,
    // the Store installed a newer healthy version while the install ran, and it was left in place
    pub store_updated_meanwhile: bool,
}
