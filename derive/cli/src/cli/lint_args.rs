use std::path::PathBuf;

use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct LintArgs {
    /// The game executable to read
    #[arg(long)]
    pub exe: PathBuf,
    /// Directory with the recipe `.hcl` files
    #[arg(long, default_value = "recipes")]
    pub recipes: PathBuf,
    /// File that receives the Markdown report; without it the report goes to standard output
    #[arg(long)]
    pub report: Option<PathBuf>,
}

impl LintArgs {
    pub fn new(exe: PathBuf, recipes: PathBuf, report: Option<PathBuf>) -> Self {
        Self { exe, recipes, report }
    }
}
