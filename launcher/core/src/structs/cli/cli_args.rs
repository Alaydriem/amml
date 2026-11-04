use super::CliCommand;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "amml-launcher",
    about = "The AMML launcher core: registries, instances, mods, updates, data and launch"
)]
pub struct CliArgs {
    // an AMML root other than `%LOCALAPPDATA%\amml`; for tests
    #[arg(long, hide = true)]
    pub root: Option<PathBuf>,
    // a registry URL that stands in for the official one; tests only, so a release build has no such argument
    #[cfg(debug_assertions)]
    #[arg(long, hide = true)]
    pub official: Option<String>,
    #[command(subcommand)]
    pub command: CliCommand,
}
