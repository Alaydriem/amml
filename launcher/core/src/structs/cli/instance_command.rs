use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum InstanceCommand {
    /// One line per instance
    List,
    /// One instance with its mods
    Show { name: String },
    /// Make an instance with AMML, the core mods and the default mods
    New {
        /// The game version, for example 1.26.60.5
        game: String,
        name: String,
        #[arg(long)]
        preview: bool,
        /// An AMML version other than the newest one that fits
        #[arg(long)]
        amml: Option<String>,
        /// Keep the game's data inside the instance instead of the shared data folder
        #[arg(long)]
        own_data: bool,
    },
    /// Delete an instance (its own data goes with it; the shared data folder stays)
    Delete {
        name: String,
        /// The instance name again
        #[arg(long)]
        confirm: String,
    },
    /// List the instances to adopt, or adopt one
    Adopt {
        name: Option<String>,
        #[arg(long)]
        own_data: bool,
    },
}
