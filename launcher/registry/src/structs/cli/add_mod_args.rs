use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct AddModArgs {
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
    pub namespace: String,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub version: String,
    /// The publish time, RFC 3339 UTC; default is the clock
    #[arg(long)]
    pub now: Option<String>,
}
