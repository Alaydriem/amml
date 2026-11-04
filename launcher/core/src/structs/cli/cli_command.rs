use super::{AmmlCommand, DataCommand, InstanceCommand, ModCommand, RegistryCommand, SettingsCommand};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    /// Registry status, search and details
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
    /// List, show, make, delete and adopt instances
    Instance {
        #[command(subcommand)]
        command: InstanceCommand,
    },
    /// Update AMML in an instance
    Amml {
        #[command(subcommand)]
        command: AmmlCommand,
    },
    /// Add, update, turn off, turn on and remove mods
    Mod {
        #[command(subcommand)]
        command: ModCommand,
    },
    /// Report which mods and which AMML have a newer release
    Updates { instance: String },
    /// Find world folders and copy them into the shared data folder
    Data {
        #[command(subcommand)]
        command: DataCommand,
    },
    /// Start an instance
    Launch {
        instance: String,
        /// Seconds to wait for the game window (default 120)
        #[arg(long)]
        window_timeout: Option<u64>,
    },
    /// Show and change the launcher settings
    Settings {
        #[command(subcommand)]
        command: SettingsCommand,
    },
}
