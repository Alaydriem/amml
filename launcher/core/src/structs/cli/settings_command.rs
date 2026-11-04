use super::PreviewSwitch;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum SettingsCommand {
    /// The current settings
    Show,
    /// Add a registry (https URL)
    AddRegistry { url: String },
    /// Remove a registry (not the official one)
    RemoveRegistry { url: String },
    /// Show or hide preview game versions
    Preview { switch: PreviewSwitch },
}
