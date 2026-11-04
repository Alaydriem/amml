use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ModCommand {
    /// Install a mod from the registries (`@namespace/name`) or from an https URL
    Add {
        instance: String,
        target: String,
        /// Allow a zip that no registry lists
        #[arg(long)]
        unofficial_ok: bool,
    },
    /// Install the newer release of one mod, or of every mod that has one
    Update {
        instance: String,
        #[arg(required_unless_present = "all", conflicts_with = "all")]
        id: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Turn a mod off
    Off { instance: String, id: String },
    /// Turn a mod on
    On { instance: String, id: String },
    /// Delete a mod from the instance
    Remove { instance: String, id: String },
}
