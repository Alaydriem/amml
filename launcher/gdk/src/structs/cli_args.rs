use super::CliCommand;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "amml-gdk", about = "Installs Minecraft Bedrock versions into the AMML version store")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CliCommand,
}
