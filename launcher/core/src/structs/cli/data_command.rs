use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum DataCommand {
    /// The world folders that can be copied
    Candidates,
    /// Copy a world folder into the shared data folder
    Copy { source: PathBuf },
}
