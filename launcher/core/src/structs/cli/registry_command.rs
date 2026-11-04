use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum RegistryCommand {
    /// The state of every registry
    Status,
    /// Find mods by id, title or tag
    Search { query: String },
    /// Show one mod
    Show { id: String },
}
