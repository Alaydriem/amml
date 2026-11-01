use std::path::PathBuf;

use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct SignatureArgs {
    /// The game executable to read
    #[arg(long)]
    pub exe: PathBuf,
    /// Directory with the recipe `.hcl` files
    #[arg(long, default_value = "recipes")]
    pub recipes: PathBuf,
    /// File that receives the signatures; without it they go to standard output unless `--compare` is given
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// A signature file of another build; each entry is reported as same, changed or unknown against it
    #[arg(long)]
    pub compare: Option<PathBuf>,
}

impl SignatureArgs {
    pub fn new(exe: PathBuf, recipes: PathBuf, out: Option<PathBuf>, compare: Option<PathBuf>) -> Self {
        Self { exe, recipes, out, compare }
    }
}
