use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct LintManifestArgs {
    /// The mod amml.hcl; the icon path is read relative to its folder
    pub manifest: PathBuf,
    /// When given, the manifest version must equal it
    #[arg(long)]
    pub version: Option<String>,
    /// When given, the manifest namespace must equal it
    #[arg(long)]
    pub namespace: Option<String>,
    /// When given, the manifest name must equal it
    #[arg(long)]
    pub name: Option<String>,
}
