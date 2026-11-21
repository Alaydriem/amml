use std::path::PathBuf;

use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct CompatArgs {
    /// The game executable to read
    #[arg(long)]
    pub exe: PathBuf,
    /// Mods root: folders `@<namespace>/<name>/amml.hcl`
    #[arg(long)]
    pub mods: PathBuf,
    /// Directory with the derived `<BuildId>.hcl` data files
    #[arg(long, default_value = "data")]
    pub data: PathBuf,
}

impl CompatArgs {
    pub fn new(exe: PathBuf, mods: PathBuf, data: PathBuf) -> Self {
        Self { exe, mods, data }
    }
}
