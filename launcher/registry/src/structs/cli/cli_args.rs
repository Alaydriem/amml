use clap::Parser;

use super::Command;

#[derive(Debug, Parser)]
#[command(name = "amml-registry", about = "Maintains an AMML registry tree")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Command,
}
