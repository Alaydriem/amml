use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum CliCommand {
    /// Lists the catalogue versions, newest first, and marks the registered and stored ones
    List {
        #[arg(long)]
        preview: bool,
    },
    /// Puts one game version into the version store
    Install {
        version: String,
        #[arg(long)]
        preview: bool,
    },
    /// Makes a loose instance folder from a stored version
    Instance { version: String, name: String },
    /// Stores the game files of an existing instance as a version
    Seed { instance_dir: PathBuf },
    /// Puts the Store version back after an install that was stopped
    Recover,
    /// Shows the registered version, Developer Mode, the version store and the install journal
    Status,
}

impl CliCommand {
    // only list and install read the catalogue
    pub fn needs_catalogue(&self) -> bool {
        matches!(self, Self::List { .. } | Self::Install { .. })
    }
}
