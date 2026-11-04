use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum AmmlCommand {
    /// Install the newest AMML that fits the instance's game, or the version given
    Update {
        instance: String,
        #[arg(long)]
        to: Option<String>,
    },
}
