use std::path::PathBuf;

use clap::Args;

use super::InspectOp;

#[derive(Args, Clone, Debug)]
pub struct InspectArgs {
    /// The game executable to read; accepted before or after the operation
    #[arg(long, global = true)]
    pub exe: Option<PathBuf>,
    #[command(subcommand)]
    pub op: InspectOp,
}

impl InspectArgs {
    pub fn new(exe: PathBuf, op: InspectOp) -> Self {
        Self { exe: Some(exe), op }
    }
}
