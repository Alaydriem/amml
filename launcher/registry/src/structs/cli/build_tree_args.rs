use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct BuildTreeArgs {
    /// The registry tree folder (holds mods/, icons/, registry.json)
    #[arg(long)]
    pub tree: PathBuf,
    /// The registry name, used when the tree has no registry.json yet
    #[arg(long, default_value = "Official")]
    pub name: String,
    /// The allow list; when given, core and default follow it
    #[arg(long)]
    pub allow: Option<PathBuf>,
    /// New publish time; default keeps the one in registry.json
    #[arg(long)]
    pub published: Option<String>,
}
