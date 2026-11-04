use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct AddAmmlArgs {
    /// The registry tree folder (holds mods/, icons/, registry.json)
    #[arg(long)]
    pub tree: PathBuf,
    /// The allow list, mods.hcl
    #[arg(long)]
    pub allow: PathBuf,
    /// The repo that made the release, owner/name
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub version: String,
    #[arg(long)]
    pub api_version: u32,
    /// Tested game versions, comma separated
    #[arg(long, value_delimiter = ',')]
    pub game_versions: Vec<String>,
    /// The publish time, RFC 3339 UTC; default is the clock
    #[arg(long)]
    pub now: Option<String>,
}
